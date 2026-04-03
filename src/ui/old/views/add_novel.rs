use iced::{
    Subscription, Task,
    widget::{button, column, row, text, text_input},
};
use iced_aw::number_input;

use crate::{
    db::index::{Index, tags::MangaTag},
    ui::{
        AppState,
        message::Message,
        views::{View, ViewMessage, novel_list::MangaListView},
    },
};

#[derive(Debug, Clone)]
pub struct AddNovelView {
    title: String,
    release_date: i32,
}

#[derive(Debug, Clone)]
pub enum AddMangaMessage {
    AddNovel,
    UpdateTitle(String),
    UpdateReleaseDate(i32),
    SavedNovel,
}

impl From<AddMangaMessage> for Message {
    fn from(m: AddMangaMessage) -> Self {
        Message::ViewMessage(ViewMessage::AddManga(m))
    }
}

impl AddNovelView {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            release_date: 0,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::none()
    }

    pub fn on_enter(_: &mut AppState) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self, _: &AppState) -> iced::Element<'_, Message> {
        column![
            text_input("Title", &self.title).on_input(|s| AddMangaMessage::UpdateTitle(s).into()),
            row![
                text("Release Date: "),
                number_input(&self.release_date, .., |v| {
                    AddMangaMessage::UpdateReleaseDate(v).into()
                })
            ],
            button(text("Add Novel")).on_press(AddMangaMessage::AddNovel.into())
        ]
        .into()
    }

    pub fn update(m: AddMangaMessage, state: &mut AppState) -> Task<Message> {
        if let View::AddNovel(v) = &mut state.view {
            match m {
                AddMangaMessage::AddNovel => {
                    if let Some(repositories) = &state.repositories {
                        let repositories = repositories.clone();
                        let novel: Index<MangaTag> = Index::new_signed(
                            v.title.clone(),
                            v.release_date,
                            &state.config.private_key(),
                        );
                        return Task::future(async move {
                            repositories.index().add_index(novel).await.unwrap();
                            AddMangaMessage::SavedNovel.into()
                        });
                    }
                }
                AddMangaMessage::UpdateTitle(title) => {
                    v.title = title;
                }
                AddMangaMessage::UpdateReleaseDate(i) => {
                    v.release_date = i;
                }
                AddMangaMessage::SavedNovel => {
                    v.title = String::new();
                    return Task::done(Message::BackHistory);
                }
            }
        }
        Task::none()
    }
}
