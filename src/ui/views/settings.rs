use iced::{
    Subscription, Task,
    widget::{button, checkbox, column, container, row, text, text_input, tooltip},
};
use tracing::info;

use crate::{
    config::AkarekoConfig,
    db::user::{TrustLevel, User},
    types::{String8, Timestamp},
    ui::{
        AppState,
        components::toast::{Toast, ToastType},
        message::Message,
        views::{View, ViewMessage},
    },
};

#[derive(Debug, Clone)]
pub struct SettingsView {
    config: AkarekoConfig,
    old_name: String,
    new_name: String,
    dirty: bool,
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    UpdateRelay(bool),
    UpdatedDevMode(bool),
    SaveConfig,
    SavedConfig(AkarekoConfig),
    UpdateName(String),
    PublishName,
}

impl From<SettingsMessage> for Message {
    fn from(m: SettingsMessage) -> Self {
        Message::ViewMessage(ViewMessage::Settings(m))
    }
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            config: AkarekoConfig::default(),
            old_name: String::new(),
            new_name: String::new(),
            dirty: false,
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::none()
    }

    pub fn on_enter(state: &mut AppState) -> Task<Message> {
        if let View::Settings(v) = &mut state.view {
            v.config = state.config.clone();
        }
        // TODO: Load from DB old_name
        Task::none()
    }

    pub fn view(&self, _: &AppState) -> iced::Element<'_, Message> {
        let pub_key = self.config.public_key().to_base64();

        let priv_key = self.config.private_key().to_base64();

        let save_message = if self.dirty {
            Some(SettingsMessage::SaveConfig.into())
        } else {
            None
        };

        column![
            row![text("Public Key: "), text(pub_key)],
            row![text("Private Key: "), text(priv_key)],
            row![
                text_input(&self.old_name, &self.new_name)
                    .on_input(|s| SettingsMessage::UpdateName(s).into()),
                button("Publish").on_press(SettingsMessage::PublishName.into())
            ],
            tooltip(
                row![
                    text("Relay: "),
                    checkbox(self.config.is_relay())
                        .on_toggle(|b| { SettingsMessage::UpdateRelay(b).into() }),
                ],
                container("Enables other users to query your node for content")
                    .padding(10)
                    .style(container::rounded_box),
                tooltip::Position::FollowCursor
            ),
            tooltip(
                row![
                    text("Dev Mode: "),
                    checkbox(self.config.dev_mode())
                        .on_toggle(|b| { SettingsMessage::UpdatedDevMode(b).into() }),
                ],
                container("Enables adding content")
                    .padding(10)
                    .style(container::rounded_box),
                tooltip::Position::FollowCursor
            ),
            button(text("Save")).on_press_maybe(save_message),
        ]
        .into()
    }

    pub fn update(m: SettingsMessage, state: &mut AppState) -> Task<Message> {
        if let View::Settings(v) = &mut state.view {
            match m {
                SettingsMessage::UpdateRelay(is_relay) => {
                    v.dirty = true;
                    v.config.set_is_relay(is_relay)
                }
                SettingsMessage::UpdatedDevMode(dev_mode) => {
                    v.dirty = true;
                    v.config.set_dev_mode(dev_mode)
                }
                SettingsMessage::UpdateName(s) => {
                    v.dirty = true;
                    v.new_name = s;
                }
                SettingsMessage::PublishName => {
                    if let Some(repositories) = state.repositories.as_ref() {
                        let repositories = repositories.clone();
                        v.old_name = std::mem::take(&mut v.new_name);

                        let mut new_user = User::new_signed(
                            String8::new(v.old_name.clone()).unwrap(),
                            Timestamp::now(),
                            state.config.private_key(),
                            state.config.eepsite_address().clone(),
                        );
                        new_user.set_trust(TrustLevel::Ignore);

                        return Task::future(async move {
                            match repositories.user().upsert_user(new_user).await {
                                Ok(_) => Message::PostToast(Toast {
                                    title: "Username published".into(),
                                    body: "Your username has been published to the network".into(),
                                    ty: ToastType::Info,
                                }),
                                Err(e) => Message::PostToast(Toast {
                                    title: "Error publishing username".into(),
                                    body: format!("{}", e),
                                    ty: ToastType::Error,
                                }),
                            }
                        });
                    }

                    return Task::done(Message::PostToast(Toast {
                        title: "Error publishing username".into(),
                        body: "Database not initialized".into(),
                        ty: ToastType::Error,
                    }));
                }
                SettingsMessage::SaveConfig => {
                    let config_to_save = v.config.clone();
                    let server_config = state.server_config.clone();
                    return Task::future(async move {
                        match config_to_save.save().await {
                            Ok(_) => {}
                            Err(e) => {
                                Message::PostToast(Toast {
                                    title: "Error saving settings".into(),
                                    body: format!("{}", e),
                                    ty: ToastType::Error,
                                });
                            }
                        }

                        let mut config = server_config.write().await;
                        *config = config_to_save.clone();

                        info!("Updated server config");
                        SettingsMessage::SavedConfig(config_to_save).into()
                    });
                }
                SettingsMessage::SavedConfig(c) => {
                    state.config = c;
                    v.dirty = false;
                }
            }
        }
        Task::none()
    }
}
