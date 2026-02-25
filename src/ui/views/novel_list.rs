use iced::{
    Subscription, Task,
    widget::{Column, button, text},
};
use tracing::error;

use crate::{
    db::index::{Index, tags::MangaTag},
    ui::{
        AppState, Message,
        components::toast::Toast,
        views::{View, ViewMessage, add_novel::AddNovelView, novel::NovelView},
    },
};

#[derive(Debug, Clone)]
pub struct MangaListView {
    novels: Vec<Index<MangaTag>>,
}

#[derive(Debug, Clone)]
pub enum NovelListMessage {
    LoadedNovels(Vec<Index<MangaTag>>),
}

impl From<NovelListMessage> for Message {
    fn from(msg: NovelListMessage) -> Message {
        Message::ViewMessage(ViewMessage::NovelList(msg))
    }
}

impl MangaListView {
    pub fn new() -> Self {
        Self { novels: vec![] }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::none()
    }

    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        if let Some(repositories) = &state.repositories {
            let repositories = repositories.clone();

            return Task::future(async move {
                let novels = match repositories.index().await.get_all_indexes(0, None).await {
                    Ok(novels) => novels,
                    Err(e) => {
                        error!("Failed to get all indexes: {}", e);
                        return Message::PostToast(Toast::error(
                            "Failed to get all indexes".into(),
                            e.to_string(),
                        ));
                    }
                };
                NovelListMessage::LoadedNovels(novels).into()
            });
        }
        Task::none()
    }

    pub fn view(&self, state: &AppState) -> iced::Element<'_, Message> {
        let mut column: Vec<iced::Element<Message>> = vec![text("Novels").into()];

        if state.config.dev_mode() {
            column.push(
                button(text("Add Novel"))
                    .on_press(Message::ChangeView(View::AddNovel(AddNovelView::new())))
                    .into(),
            );
        }

        for novel in self.novels.iter() {
            column.push(
                button(text(novel.title().clone()))
                    .on_press(Message::ChangeView(View::Novel(NovelView::new(
                        novel.clone(),
                    ))))
                    .into(),
            );
        }

        Column::from_vec(column).into()
    }

    pub fn update(m: NovelListMessage, state: &mut AppState) -> Task<Message> {
        if let View::NovelList(v) = &mut state.view {
            match m {
                NovelListMessage::LoadedNovels(novels) => {
                    v.novels = novels;
                }
            }
        }
        Task::none()
    }
}
