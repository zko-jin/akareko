use iced::{
    Subscription, Task,
    widget::{Column, button, center, checkbox, column, row, text, text_editor, text_input},
};
use iced_aw::number_input;

use crate::{
    db::{
        Magnet,
        index::{
            Index,
            content::{Content, ContentEntry},
            tags::{MangaChapter, MangaTag},
        },
    },
    helpers::{Language, now_timestamp},
    ui::{
        AppState,
        message::Message,
        views::{View, ViewMessage, novel::MangaView},
    },
};

#[derive(Debug, Clone, Default)]
struct ContentEntryValues {
    title: String,
    path: String,
    enumeration: f32,
    end: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct AddMangaChapterView {
    novel: Index<MangaTag>,
    magnet: String,
    entries: Vec<ContentEntryValues>,
}

#[derive(Debug, Clone)]
pub enum AddMangaChapterMessage {
    AddContent,

    UpdateTitle(String, usize),
    UpdateEnumeration(f32, usize),
    UpdateEnd(Option<f32>, usize),
    UpdatePath(String, usize),
    AddEntry,
    RemoveEntry(usize),

    UpdateMagnet(String),
    SavedContent,
}

impl From<AddMangaChapterMessage> for Message {
    fn from(m: AddMangaChapterMessage) -> Self {
        Message::ViewMessage(ViewMessage::AddMangaChapter(m))
    }
}

impl AddMangaChapterView {
    pub fn new(novel: Index<MangaTag>) -> Self {
        Self {
            novel,
            magnet: String::new(),
            entries: vec![],
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::none()
    }

    pub fn on_enter(_: &mut AppState) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self, _: &AppState) -> iced::Element<'_, Message> {
        let entries = self
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let mut enum_row: iced::widget::Row<'_, Message> = row![
                    number_input(&e.enumeration, 0.0.., move |v| {
                        Message::ViewMessage(ViewMessage::AddMangaChapter(
                            AddMangaChapterMessage::UpdateEnumeration(v, i),
                        ))
                    }),
                    text("Is Volume: "),
                    checkbox(e.end.is_some()).on_toggle(move |b| if b {
                        AddMangaChapterMessage::UpdateEnd(Some(e.enumeration), i).into()
                    } else {
                        AddMangaChapterMessage::UpdateEnd(None, i).into()
                    }),
                ];

                if let Some(end) = e.end {
                    enum_row = enum_row.push(text("Ends at: ")).push(number_input(
                        &end,
                        0.0..,
                        move |v| AddMangaChapterMessage::UpdateEnd(Some(v), i).into(),
                    ));
                }

                column![
                    text_input("Title", &e.title)
                        .on_input(move |s| AddMangaChapterMessage::UpdateTitle(s, i).into())
                        .width(iced::Length::Fill),
                    text_input("Path", &e.path)
                        .on_input(move |s| AddMangaChapterMessage::UpdatePath(s, i).into())
                        .width(iced::Length::Fill),
                    enum_row,
                ]
                .into()
            })
            .collect();

        let entries_column = Column::from_vec(entries).width(iced::Length::Fill);

        column![
            text_input("Magnet", &self.magnet)
                .on_input(|s| AddMangaChapterMessage::UpdateMagnet(s).into()),
            center(row![
                button(text("+")).on_press(AddMangaChapterMessage::AddEntry.into()),
                button(text("-")).on_press_maybe(match self.entries.len() {
                    0 => None,
                    _ => Some(AddMangaChapterMessage::RemoveEntry(self.entries.len() - 1).into()),
                }),
            ],)
            .height(iced::Length::Shrink),
            entries_column,
            button(text("Add Chapter")).on_press(AddMangaChapterMessage::AddContent.into())
        ]
        .into()
    }

    pub fn update(m: AddMangaChapterMessage, state: &mut AppState) -> Task<Message> {
        if let View::AddChapter(v) = &mut state.view {
            match m {
                AddMangaChapterMessage::AddContent => {
                    if let Some(repositories) = &state.repositories {
                        let index_hash = v.novel.hash().clone();

                        let entries: Vec<ContentEntry<MangaTag>> = v
                            .entries
                            .iter()
                            .map(|e| ContentEntry {
                                title: e.title.clone(),
                                enumeration: e.enumeration,
                                end: e.end,
                                path: e.path.clone(),
                                extra_metadata: MangaChapter::new(Language::English),
                                progress: 0.0,
                            })
                            .collect();

                        let chapter = Content::new_signed(
                            state.config.public_key().clone(),
                            index_hash,
                            now_timestamp(),
                            Magnet(v.magnet.clone()),
                            entries,
                            state.config.private_key(),
                        );

                        let repositories = repositories.clone();
                        return Task::future(async move {
                            match repositories.index().add_content(chapter).await {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("Error adding chapter: {}", e);
                                }
                            }
                            AddMangaChapterMessage::SavedContent.into()
                        });
                    }
                }
                AddMangaChapterMessage::UpdateTitle(title, i) => {
                    v.entries[i].title = title;
                }
                AddMangaChapterMessage::UpdateEnumeration(enumeration, i) => {
                    v.entries[i].enumeration = enumeration;
                }
                AddMangaChapterMessage::UpdatePath(path, i) => {
                    v.entries[i].path = path;
                }
                AddMangaChapterMessage::UpdateMagnet(magnet) => {
                    v.magnet = magnet;
                }
                AddMangaChapterMessage::UpdateEnd(e, i) => {
                    v.entries[i].end = e.map(|n| {
                        if n > v.entries[i].enumeration {
                            n
                        } else {
                            v.entries[i].enumeration
                        }
                    });
                }
                AddMangaChapterMessage::AddEntry => {
                    v.entries.push(ContentEntryValues::default());
                }
                AddMangaChapterMessage::RemoveEntry(i) => {
                    v.entries.remove(i);
                }
                AddMangaChapterMessage::SavedContent => {
                    v.entries = vec![];
                    v.magnet = String::new();
                    return Task::done(Message::BackHistory);
                }
            }
        }
        Task::none()
    }
}
