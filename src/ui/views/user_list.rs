use iced::{
    Subscription, Task,
    widget::{Column, row, text},
};

use crate::{
    db::user::User,
    ui::{
        AppState, Message,
        views::{View, ViewMessage},
    },
};

#[derive(Debug, Clone)]
pub struct UserListView {
    users: Vec<User>,
}

#[derive(Debug, Clone)]
pub enum UserListMessage {
    LoadedUsers(Vec<User>),
}

impl From<UserListMessage> for Message {
    fn from(msg: UserListMessage) -> Message {
        Message::ViewMessage(ViewMessage::UserList(msg))
    }
}

impl UserListView {
    pub fn new() -> Self {
        Self { users: vec![] }
    }

    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        if let Some(repositories) = &state.repositories {
            let repositories = repositories.clone();

            return Task::future(async move {
                let users = repositories.user().await.get_all_users().await;
                UserListMessage::LoadedUsers(users).into()
            });
        }
        Task::none()
    }

    pub fn view(&self, _: &AppState) -> iced::Element<'_, Message> {
        let mut column: Vec<iced::Element<Message>> = vec![text("Users").into()];

        for user in self.users.iter() {
            column.push(
                row![
                    text(user.name().clone() + " | "),
                    text(user.pub_key().to_base64() + " | "),
                    text(user.address().to_string()),
                ]
                .into(),
            );
        }

        Column::from_vec(column).into()
    }

    pub fn update(m: UserListMessage, state: &mut AppState) -> Task<Message> {
        if let View::UserList(v) = &mut state.view {
            match m {
                UserListMessage::LoadedUsers(users) => {
                    v.users = users;
                }
            }
        }
        Task::none()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::none()
    }
}
