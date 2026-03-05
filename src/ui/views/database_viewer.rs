use std::collections::HashSet;

use iced::{
    Subscription, Task,
    widget::{
        Column, Row, Text, button, row, scrollable, table, text,
        text_editor::{self, Content},
    },
};
use surrealdb_types::{ToSql, Value};

use crate::{
    db::{
        comments::{Post, Topic},
        event::{Event, get_paginated_events, make_event_filter},
        user::User,
    },
    helpers::now_timestamp,
    ui::{
        AppState,
        components::toast::{Toast, ToastType},
        message::Message,
        views::{View, ViewMessage},
    },
};

#[derive(Clone, Debug)]
enum Tab {
    FreeQuery(Content, String),
    Users,
    Events {
        events: Vec<Event>,
        cur_page: usize,
        total_pages: usize,
    },
}

#[derive(Debug, Clone)]
struct TableView<T> {
    events: Vec<T>,
    cur_page: usize,
    total_pages: usize,
}

#[derive(Debug, Clone)]
pub struct DatabaseViewerView {
    tab: Tab,
}

#[derive(Debug, Clone)]
pub enum DatabaseViewerMessage {
    ChangeTab(Tab),

    ExecuteQuery(String),
    QueryResult(String),
    EditQuery(text_editor::Action),

    LoadEvents {
        events: Vec<Event>,
        total_pages: usize,
    },
}

impl From<DatabaseViewerMessage> for Message {
    fn from(msg: DatabaseViewerMessage) -> Message {
        Message::ViewMessage(ViewMessage::DatabaseViewer(msg))
    }
}

impl DatabaseViewerView {
    pub fn new() -> Self {
        Self {
            tab: Tab::FreeQuery(Content::new(), String::new()),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::none()
    }

    pub fn on_enter(_state: &mut AppState) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self, _state: &AppState) -> iced::Element<'_, Message> {
        let mut column = Column::new();
        column = column.push(row![
            button("Users").on_press(DatabaseViewerMessage::ChangeTab(Tab::Users).into()),
            button("Events").on_press(
                DatabaseViewerMessage::ChangeTab(Tab::Events {
                    events: vec![],
                    cur_page: 1,
                    total_pages: 0,
                })
                .into()
            ),
            button("Free Query").on_press(
                DatabaseViewerMessage::ChangeTab(Tab::FreeQuery(Content::new(), String::new(),))
                    .into()
            ),
        ]);

        match &self.tab {
            Tab::FreeQuery(content, results) => {
                column = column.push(
                    iced::widget::text_editor(content)
                        .on_action(|a| DatabaseViewerMessage::EditQuery(a).into()),
                );
                column = column.push(button("Execute").on_press(
                    DatabaseViewerMessage::ExecuteQuery(content.text().to_string()).into(),
                ));

                column = column.push(scrollable(Text::new(results)));
            }
            Tab::Events {
                events,
                cur_page,
                total_pages,
            } => {
                // id
                // timestamp: Timestamp,
                //  event_type: EventType,
                //  topic: Topic,
                let table = table(
                    vec![
                        table::column("Id", |e: &Event| text(e.topic.as_base64())),
                        table::column("Timestamp", |e: &Event| text(e.timestamp.to_string())),
                        table::column("Type", |e: &Event| text(e.event_type.as_str())),
                    ],
                    events,
                );

                column = column.push(text(format!("{} / {}", cur_page, total_pages)));
                column = column.push(scrollable(table));
            }
            _ => todo!(),
        }
        column.into()
    }

    pub fn update(m: DatabaseViewerMessage, state: &mut AppState) -> Task<Message> {
        if let View::DatabaseViewer(v) = &mut state.view {
            match m {
                DatabaseViewerMessage::ExecuteQuery(query) => {
                    if let Tab::FreeQuery(_, ref mut results) = v.tab {
                        match state.repositories.clone() {
                            Some(repositories) => {
                                return Task::future(async move {
                                    let res: Result<Vec<Value>, _> =
                                        repositories.db.query(&query).await.unwrap().take(0);
                                    let str = match res {
                                        Ok(r) => {
                                            let mut str = String::new();
                                            for e in r {
                                                str.push_str(&e.to_sql_pretty());
                                            }
                                            str
                                        }
                                        Err(e) => e.to_string(),
                                    };
                                    DatabaseViewerMessage::QueryResult(str).into()
                                });
                            }
                            None => *results = "Repository not initialized".to_string(),
                        };
                    }
                }
                DatabaseViewerMessage::QueryResult(s) => {
                    if let Tab::FreeQuery(_, ref mut results) = v.tab {
                        *results = s;
                    }
                }
                DatabaseViewerMessage::EditQuery(a) => {
                    if let Tab::FreeQuery(ref mut content, _) = v.tab {
                        content.perform(a);
                    }
                }
                DatabaseViewerMessage::ChangeTab(tab) => {
                    v.tab = tab.clone();
                    match tab {
                        Tab::FreeQuery(_, _) => {}
                        Tab::Events { .. } => {
                            let Some(repo) = state.repositories.clone() else {
                                return Task::done(
                                    Toast::error(
                                        "Repository not loaded".to_string(),
                                        "".to_string(),
                                    )
                                    .into(),
                                );
                            };

                            return Task::future(async move {
                                let (events, total_pages) =
                                    get_paginated_events(1, 50, &repo.db).await.unwrap();

                                DatabaseViewerMessage::LoadEvents {
                                    events,
                                    total_pages,
                                }
                                .into()
                            });
                        }
                        Tab::Users => todo!(),
                    }
                }
                DatabaseViewerMessage::LoadEvents {
                    events,
                    total_pages,
                } => {
                    if let Tab::Events {
                        events: ref mut events_ref,
                        total_pages: ref mut ref_total_pages,
                        ..
                    } = v.tab
                    {
                        *events_ref = events;
                        *ref_total_pages = total_pages;
                    }
                }
            }
        }
        Task::none()
    }
}
