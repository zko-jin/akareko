use std::collections::HashSet;

use iced::{
    Subscription, Task,
    widget::{
        Column, Row, Text, button, scrollable, text,
        text_editor::{self, Content},
    },
};
use surrealdb_types::{ToSql, Value};

use crate::{
    db::{
        comments::{Post, Topic},
        event::Event,
        user::User,
    },
    helpers::now_timestamp,
    ui::{
        AppState, Message,
        components::toast::{Toast, ToastType},
        views::{View, ViewMessage},
    },
};

#[derive(Clone, Debug)]
enum Table {
    Events(Vec<Event>),
}

#[derive(Debug, Clone)]
pub struct DatabaseViewerView {
    results: String,

    content: Content,
}

#[derive(Debug, Clone)]
pub enum DatabaseViewerMessage {
    ExecuteQuery(String),
    QueryResult(String),
    EditQuery(text_editor::Action),
}

impl From<DatabaseViewerMessage> for Message {
    fn from(msg: DatabaseViewerMessage) -> Message {
        Message::ViewMessage(ViewMessage::DatabaseViewer(msg))
    }
}

impl DatabaseViewerView {
    pub fn new() -> Self {
        Self {
            results: String::new(),
            content: Content::new(),
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
        column = column.push(
            iced::widget::text_editor(&self.content)
                .on_action(|a| DatabaseViewerMessage::EditQuery(a).into()),
        );
        column =
            column.push(button("Execute").on_press(
                DatabaseViewerMessage::ExecuteQuery(self.content.text().to_string()).into(),
            ));

        column = column.push(scrollable(Text::new(&self.results)));
        column.into()
    }

    pub fn update(m: DatabaseViewerMessage, state: &mut AppState) -> Task<Message> {
        if let View::DatabaseViewer(v) = &mut state.view {
            match m {
                DatabaseViewerMessage::ExecuteQuery(query) => {
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
                        None => v.results = "Repository not initialized".to_string(),
                    };
                }
                DatabaseViewerMessage::QueryResult(s) => {
                    v.results = s;
                }
                DatabaseViewerMessage::EditQuery(a) => {
                    v.content.perform(a);
                }
            }
        }
        Task::none()
    }
}
