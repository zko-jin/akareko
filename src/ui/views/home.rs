use iced::{
    Subscription, Task,
    widget::{button, column, text},
};

use crate::ui::{
    AppState, Message,
    components::{
        modal::{Modal, add_who::AddWhoModal},
        toast::Toast,
    },
    views::{
        MangaListView, View, database_viewer::DatabaseViewerView, settings::SettingsView,
        user_list::UserListView,
    },
};

#[derive(Debug, Clone)]
pub struct HomeView {}

#[derive(Debug, Clone)]
pub enum HomeMessage {}

impl HomeView {
    pub fn new() -> Self {
        Self {}
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::none()
    }

    pub fn on_enter(_: &mut AppState) -> Task<Message> {
        Task::none()
    }

    pub fn view(&self, _: &AppState) -> iced::Element<'_, Message> {
        column![
            button(text("Novels"))
                .on_press(Message::ChangeView(View::NovelList(MangaListView::new()))),
            button(text("Settings"))
                .on_press(Message::ChangeView(View::Settings(SettingsView::new()))),
            button(text("Add user"))
                .on_press(Message::OpenModal(Modal::AddWho(AddWhoModal::new()))),
            button(text("SaveTorrent")).on_press(Message::SaveTorrent),
            button(text("User List"))
                .on_press(Message::ChangeView(View::UserList(UserListView::new()))),
            button(text("Database Viewer")).on_press(Message::ChangeView(View::DatabaseViewer(
                DatabaseViewerView::new(),
            ))),
        ]
        .into()
    }

    pub fn update(_: HomeMessage, _: &mut AppState) -> Task<Message> {
        Task::none()
    }
}
