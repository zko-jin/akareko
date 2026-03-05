use iced::{
    Subscription, Task,
    widget::{Column, checkbox, row, text},
};
use tracing::error;

use crate::{
    db::{
        FullSyncTarget,
        schedule::{Schedule, ScheduleType},
        user::User,
    },
    hash::PublicKey,
    ui::{
        AppState,
        components::toast::Toast,
        message::Message,
        views::{View, ViewMessage},
    },
};

#[derive(Debug, Clone)]
pub struct UserListView {
    users: Vec<User>,
    full_sync_targets: Vec<PublicKey>,
}

#[derive(Debug, Clone)]
pub enum UserListMessage {
    LoadedUsers(Vec<User>, Vec<PublicKey>),
    AddFullSyncTarget(User),
    RemoveFullSyncTarget(User),
}

impl From<UserListMessage> for Message {
    fn from(msg: UserListMessage) -> Message {
        Message::ViewMessage(ViewMessage::UserList(msg))
    }
}

impl UserListView {
    pub fn new() -> Self {
        Self {
            users: vec![],
            full_sync_targets: vec![],
        }
    }

    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        if let Some(repositories) = &state.repositories {
            let repositories = repositories.clone();

            return Task::future(async move {
                let users = repositories.user().get_all_users().await;
                let full_sync_targets = match repositories.full_sync_addresses().await {
                    Ok(targets) => targets,
                    Err(e) => {
                        error!("Failed to get full sync targets: {}", e);
                        return Toast::error(
                            "Failed to get full sync targets".into(),
                            e.to_string(),
                        )
                        .into();
                    }
                };

                UserListMessage::LoadedUsers(
                    users,
                    full_sync_targets.into_iter().map(|f| f.pub_key).collect(),
                )
                .into()
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
                    checkbox(self.full_sync_targets.contains(user.pub_key())).on_toggle(move |b| {
                        if b {
                            UserListMessage::AddFullSyncTarget(user.clone()).into()
                        } else {
                            UserListMessage::RemoveFullSyncTarget(user.clone()).into()
                        }
                    }),
                ]
                .into(),
            );
        }

        Column::from_vec(column).into()
    }

    pub fn update(m: UserListMessage, state: &mut AppState) -> Task<Message> {
        if let View::UserList(v) = &mut state.view {
            match m {
                UserListMessage::LoadedUsers(users, full_sync_targets) => {
                    v.users = users;
                    v.full_sync_targets = full_sync_targets;
                }
                UserListMessage::AddFullSyncTarget(user) => {
                    let Some(repositories) = state.repositories.clone() else {
                        return Task::none();
                    };

                    let full_sync_interval = state.config.scheduler_config().full_sync_interval;
                    v.full_sync_targets.push(user.pub_key().clone());
                    return Task::future(async move {
                        let target = FullSyncTarget::from_user(&user);
                        repositories
                            .upsert_full_sync_address(target.clone())
                            .await
                            .unwrap();

                        Message::AddSchedule(Schedule {
                            when: target.last_sync + full_sync_interval,
                            address: user.into_address(),
                            schedule_type: ScheduleType::FullSync(target.pub_key),
                            last_sync: target.last_sync,
                        })
                    });
                }
                UserListMessage::RemoveFullSyncTarget(user) => {
                    let Some(repositories) = state.repositories.clone() else {
                        return Task::none();
                    };

                    v.full_sync_targets.retain(|f| f != user.pub_key());

                    return Task::future(async move {
                        repositories
                            .remove_full_sync_address(user.pub_key().clone())
                            .await
                            .unwrap();

                        Message::RemoveSchedule(Schedule {
                            when: 0,
                            address: user.address().clone(),
                            schedule_type: ScheduleType::FullSync(user.into_pub_key()),
                            last_sync: 0,
                        })
                    });
                }
            }
        }
        Task::none()
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::none()
    }
}
