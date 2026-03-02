use iced::{
    Subscription, Task,
    widget::{button, column, text, text_input},
};

use crate::{
    db::index::{Index, tags::MangaTag},
    ui::{
        AppState, Message,
        views::{View, ViewMessage, novel_list::MangaListView},
    },
};

#[derive(Debug, Clone)]
pub struct AddNovelView {
    title: String,
}

#[derive(Debug, Clone)]
pub enum AddNovelMessage {
    AddNovel,
    UpdateTitle(String),
    SavedNovel,
}

impl From<AddNovelMessage> for Message {
    fn from(m: AddNovelMessage) -> Self {
        Message::ViewMessage(ViewMessage::AddNovel(m))
    }
}

impl AddNovelView {
    pub fn new() -> Self {
        Self {
            title: String::new(),
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
            text_input("Title", &self.title).on_input(|s| AddNovelMessage::UpdateTitle(s).into()),
            button(text("Add Novel")).on_press(AddNovelMessage::AddNovel.into())
        ]
        .into()
    }

    pub fn update(m: AddNovelMessage, state: &mut AppState) -> Task<Message> {
        if let View::AddNovel(v) = &mut state.view {
            match m {
                AddNovelMessage::AddNovel => {
                    if let Some(repositories) = &state.repositories {
                        let repositories = repositories.clone();
                        let novel: Index<MangaTag> =
                            Index::new_signed(v.title.clone(), 0, &state.config.private_key());
                        return Task::future(async move {
                            repositories.index().add_index(novel).await.unwrap();
                            AddNovelMessage::SavedNovel.into()
                        });
                    }
                }
                AddNovelMessage::UpdateTitle(title) => {
                    v.title = title;
                }
                AddNovelMessage::SavedNovel => {
                    v.title = String::new();
                    return Task::done(Message::ChangeView(View::NovelList(MangaListView::new())));
                }
            }
        }
        Task::none()
    }
}
