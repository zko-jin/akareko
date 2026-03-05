use iced::{
    Task,
    widget::{button, column, container, pick_list, row, text, text_input},
};
use rclite::Arc;
use tracing::{error, info};

use crate::{
    db::user::{I2PAddress, TrustLevel, User},
    errors::ClientError,
    ui::{
        AppState,
        components::{
            modal::{Modal, ModalMessage},
            toast::{Toast, ToastType},
        },
        message::Message,
    },
};

#[derive(Debug, Clone)]
pub struct AddWhoModal {
    pub i2p: String,
    pub loading: bool,
    pub user: Option<User>,
}

#[derive(Debug, Clone)]
pub enum AddWhoModalMessage {
    UpdateI2P(String),
    SearchAddress,
    GotUser(User),
    FailedGetUser(Arc<ClientError>),
    UpdateTrust(TrustLevel),
    AddUser,
    AddedUser,
}

impl From<AddWhoModalMessage> for Message {
    fn from(m: AddWhoModalMessage) -> Self {
        Message::ModalMessage(ModalMessage::AddWho(m))
    }
}

impl AddWhoModal {
    pub fn new() -> Self {
        Self {
            i2p: String::new(),
            loading: false,
            user: None,
        }
    }

    pub fn view(&self, _: &AppState) -> iced::Element<'_, Message> {
        if self.loading {
            return text("Loading...").into();
        }

        if let Some(user) = &self.user {
            return container(column![
                row![text(format!("Name: {}", user.name().clone()))],
                row![text(format!("Pub Key: {}", user.pub_key().to_base64()))],
                row![text(format!("I2P Address: {}", user.address().to_string()))],
                pick_list(TrustLevel::ALL, Some(user.trust()), |t| {
                    AddWhoModalMessage::UpdateTrust(t).into()
                }),
                button(text("Add User")).on_press(AddWhoModalMessage::AddUser.into()),
            ])
            .into();
        }

        container(column![
            row![
                text("I2P Address: "),
                text_input("I2P Address", &self.i2p)
                    .on_input(|s| AddWhoModalMessage::UpdateI2P(s).into()),
            ],
            button(text("Submit")).on_press(AddWhoModalMessage::SearchAddress.into()),
        ])
        .into()
    }

    pub fn update(m: AddWhoModalMessage, state: &mut AppState) -> Task<Message> {
        if let Some(Modal::AddWho(v)) = &mut state.modal {
            match m {
                AddWhoModalMessage::UpdateI2P(i2p) => {
                    v.i2p = i2p;
                }
                AddWhoModalMessage::UpdateTrust(trust) => {
                    if let Some(user) = &mut v.user {
                        user.set_trust(trust);
                    }
                }
                AddWhoModalMessage::SearchAddress => {
                    v.loading = true;

                    if let Some(pool) = &state.client_pool {
                        let i2p = v.i2p.clone();
                        let pool = pool.clone();
                        return Task::future(async move {
                            let mut client = pool.get_client().await;
                            let user = match client.who(&I2PAddress::new(i2p)).await {
                                Ok(user) => user,
                                Err(e) => {
                                    error!("Failed to get user: {}", e);
                                    return AddWhoModalMessage::FailedGetUser(Arc::new(e)).into();
                                }
                            };

                            info!("Got user: {:?}", user);

                            AddWhoModalMessage::GotUser(user).into()
                        });
                    }
                }
                AddWhoModalMessage::AddUser => {
                    if let Some(user) = &v.user {
                        let user = user.clone();
                        if let Some(repositories) = &state.repositories {
                            let repository = repositories.clone();
                            return Task::future(async move {
                                repository.user().upsert_user(user).await.unwrap();
                                AddWhoModalMessage::AddedUser.into()
                            });
                        }
                    }
                }
                AddWhoModalMessage::GotUser(user) => {
                    v.loading = false;
                    v.user = Some(user);
                }
                AddWhoModalMessage::AddedUser => {
                    v.user = None;
                    state.close_modal();
                }
                AddWhoModalMessage::FailedGetUser(e) => {
                    v.loading = false;
                    state.toasts.push(Toast {
                        title: "Error getting user".into(),
                        body: format!("{}", *e),
                        ty: ToastType::Error,
                    });
                }
            }
        }
        Task::none()
    }
}
