use iced::{
    Subscription, Task,
    widget::{Column, button, text},
};
use tracing::error;

use crate::{
    db::{
        follow_index::IndexFollow,
        index::{Index, tags::MangaTag},
    },
    ui::{
        AppState,
        components::toast::Toast,
        message::Message,
        views::{View, ViewMessage, add_novel::AddNovelView, novel::MangaView},
    },
};

#[derive(Debug, Clone)]
pub struct MangaListView {
    mangas: Vec<Index<MangaTag>>,
}

#[derive(Debug, Clone)]
pub enum MangaListMessage {
    LoadedMangas(Vec<Index<MangaTag>>),
}

impl From<MangaListMessage> for Message {
    fn from(msg: MangaListMessage) -> Message {
        Message::ViewMessage(ViewMessage::MangaList(msg))
    }
}

impl MangaListView {
    pub fn new() -> Self {
        Self { mangas: vec![] }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::none()
    }

    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        if let Some(repositories) = &state.repositories {
            let repositories = repositories.clone();

            return Task::future(async move {
                let novels = match repositories.index().get_all_indexes(None, None).await {
                    Ok(novels) => novels,
                    Err(e) => {
                        error!("Failed to get all indexes: {}", e);
                        return Toast::error("Failed to get all indexes", e).into();
                    }
                };
                MangaListMessage::LoadedMangas(novels).into()
            });
        }
        Task::none()
    }

    pub fn view(&self, state: &AppState) -> iced::Element<'_, Message> {
        let mut column: Vec<iced::Element<Message>> = vec![text("Mangas").into()];

        if state.config.dev_mode() {
            column.push(
                button(text("Add Manga"))
                    .on_press(Message::ChangeView(View::AddNovel(AddNovelView::new())))
                    .into(),
            );
        }

        for novel in self.mangas.iter() {
            column.push(
                button(text(novel.title().clone()))
                    .on_press(Message::ChangeView(View::Novel(MangaView::new(
                        novel.clone(),
                    ))))
                    .into(),
            );
        }

        Column::from_vec(column).into()
    }

    pub fn update(m: MangaListMessage, state: &mut AppState) -> Task<Message> {
        if let View::MangaList(v) = &mut state.view {
            match m {
                MangaListMessage::LoadedMangas(novels) => {
                    v.mangas = novels;
                }
            }
        }
        Task::none()
    }
}
