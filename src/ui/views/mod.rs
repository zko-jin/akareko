pub mod add_chapter;
pub mod add_novel;
pub mod home;
pub mod image_viewer;
pub mod novel;
pub mod novel_list;
pub mod post;
pub mod settings;
pub mod user_list;

use iced::Task;

use crate::ui::{
    AppState, Message,
    views::{
        add_chapter::{AddMangaChapterMessage, AddMangaChapterView},
        add_novel::{AddNovelMessage, AddNovelView},
        home::{HomeMessage, HomeView},
        image_viewer::{ImageViewerMessage, ImageViewerView},
        novel::{NovelMessage, NovelView},
        novel_list::{MangaListView, NovelListMessage},
        post::{PostMessage, PostView},
        settings::{SettingsMessage, SettingsView},
        user_list::{UserListMessage, UserListView},
    },
};

#[derive(Debug, Clone)]
pub enum View {
    Home(HomeView),
    NovelList(MangaListView),
    Novel(NovelView),
    AddNovel(AddNovelView),
    AddChapter(AddMangaChapterView),
    Settings(SettingsView),
    ImageViewer(ImageViewerView),
    UserList(UserListView),
    Post(PostView),
}

#[derive(Debug, Clone)]
pub enum ViewMessage {
    Home(HomeMessage),
    NovelList(NovelListMessage),
    Novel(NovelMessage),
    AddNovel(AddNovelMessage),
    AddChapter(AddMangaChapterMessage),
    Settings(SettingsMessage),
    ImageViewer(ImageViewerMessage),
    UserList(UserListMessage),
    Post(PostMessage),
}

impl View {
    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        match state.view {
            View::Home(_) => HomeView::on_enter(state),
            View::NovelList(_) => MangaListView::on_enter(state),
            View::Novel(_) => NovelView::on_enter(state),
            View::AddNovel(_) => AddNovelView::on_enter(state),
            View::AddChapter(_) => AddMangaChapterView::on_enter(state),
            View::Settings(_) => SettingsView::on_enter(state),
            View::ImageViewer(_) => ImageViewerView::on_enter(state),
            View::UserList(_) => UserListView::on_enter(state),
            View::Post(_) => PostView::on_enter(state),
        }
    }

    pub fn view(state: &AppState) -> iced::Element<'_, Message> {
        match &state.view {
            View::Home(v) => v.view(state),
            View::NovelList(v) => v.view(state),
            View::Novel(v) => v.view(state),
            View::AddNovel(v) => v.view(state),
            View::AddChapter(v) => v.view(state),
            View::Settings(v) => v.view(state),
            View::ImageViewer(v) => v.view(state),
            View::UserList(v) => v.view(state),
            View::Post(v) => v.view(state),
        }
    }

    pub fn update(message: ViewMessage, state: &mut AppState) -> Task<Message> {
        match message {
            ViewMessage::Home(m) => HomeView::update(m, state),
            ViewMessage::NovelList(m) => MangaListView::update(m, state),
            ViewMessage::Novel(m) => NovelView::update(m, state),
            ViewMessage::AddNovel(m) => AddNovelView::update(m, state),
            ViewMessage::AddChapter(m) => AddMangaChapterView::update(m, state),
            ViewMessage::Settings(m) => SettingsView::update(m, state),
            ViewMessage::ImageViewer(m) => ImageViewerView::update(m, state),
            ViewMessage::UserList(m) => UserListView::update(m, state),
            ViewMessage::Post(m) => PostView::update(m, state),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        match self {
            View::Home(v) => v.subscription(),
            View::NovelList(v) => v.subscription(),
            View::Novel(v) => v.subscription(),
            View::AddNovel(v) => v.subscription(),
            View::AddChapter(v) => v.subscription(),
            View::Settings(v) => v.subscription(),
            View::ImageViewer(v) => v.subscription(),
            View::UserList(v) => v.subscription(),
            View::Post(v) => v.subscription(),
        }
    }
}
