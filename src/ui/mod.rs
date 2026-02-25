use anawt::{AlertCategory, SettingsPack, TorrentClient, options::AnawtOptions};
use iced::{
    Length, Subscription, Task, Theme, alignment,
    widget::{Column, Container, button, column, stack, text},
    window,
};
use rclite::Arc;
use std::path::PathBuf;
use tokio::sync::{RwLock, mpsc};
use tracing::error;

use crate::{
    config::AuroraConfig,
    db::Repositories,
    server::{AuroraServer, client::AuroraClient},
    ui::{
        components::{
            modal::{Modal, ModalMessage, modal},
            toast::{Toast, ToastType, toast_worker},
        },
        views::{View, ViewMessage, home::HomeView},
    },
};

mod components;
mod icons;
mod style;
mod views;

#[derive(Debug, Clone)]
pub enum Message {
    OpenWindow,

    RepositoryLoaded(Repositories),
    ConfigLoaded(AuroraConfig),
    TorrentClientLoaded(TorrentClient),
    ClientLoaded(AuroraClient),
    DownloadTorrent { magnet: String, path: String },
    ChangeView(View),
    ViewMessage(ViewMessage),
    BackHistory,

    ToastSenderReady(mpsc::Sender<Toast>),
    PostToast(Toast),
    CloseToast(usize),

    Exchange,
    FinishExchange,

    ModalMessage(ModalMessage),
    OpenModal(Modal),
    CloseModal,

    SaveTorrent,
    Nothing,
}

#[derive(Debug, Clone)]
pub struct LiFo<T, const N: usize> {
    stack: [Option<T>; N],
    last_index: usize,
}

impl<T, const N: usize> LiFo<T, N> {
    pub fn new() -> Self {
        Self {
            stack: [const { None }; N],
            last_index: N - 1,
        }
    }

    pub fn push(&mut self, item: T) {
        self.last_index = (self.last_index + 1) % N;
        self.stack[self.last_index] = Some(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        let item = self.stack[self.last_index].take();
        if item.is_some() {
            self.last_index = (self.last_index + N - 1) % N;
        }
        item
    }

    pub fn can_pop(&self) -> bool {
        self.stack[self.last_index].is_some()
    }
}

pub struct AppState {
    repositories: Option<Repositories>,
    config: AuroraConfig,
    server_config: Arc<RwLock<AuroraConfig>>,

    view: View,
    history: LiFo<View, 10>,

    client: Option<AuroraClient>,
    torrent_client: Option<TorrentClient>,

    toast_tx: Option<mpsc::Sender<Toast>>,
    toasts: Vec<Toast>,

    exchanging: bool,

    theme: Theme,

    modal: Option<Modal>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            repositories: None,
            config: AuroraConfig::default(),
            client: None,
            server_config: Arc::new(RwLock::new(AuroraConfig::default())),
            torrent_client: None,
            view: View::Home(HomeView::new()),
            history: LiFo::new(),
            toast_tx: None,
            toasts: Vec::new(),
            exchanging: false,
            theme: Theme::CatppuccinMocha,
            modal: None,
        }
    }

    fn has_initialized(&self) -> bool {
        self.repositories.is_some() && self.client.is_some() && self.torrent_client.is_some()
    }

    pub fn theme(&self) -> Theme {
        self.theme.clone()
    }

    pub fn view(&self, _id: window::Id) -> iced::Element<'_, Message> {
        if !self.has_initialized() {
            return column![text("Loading...")].into();
        }

        let mut back = button(text("Back"));

        if self.history.can_pop() {
            back = back.on_press(Message::BackHistory);
        }

        let base = column![back, View::view(self)]
            .width(iced::Length::Fill)
            .height(iced::Length::Fill);

        let base = if self.modal.is_some() {
            modal(base, Modal::view(self), Message::CloseModal)
        } else {
            base.into()
        };

        let toasts = self
            .toasts
            .iter()
            .rev()
            .enumerate()
            .map(|(i, t)| t.view(i))
            .collect();

        stack![
            base,
            Container::new(Column::from_vec(toasts).align_x(alignment::Horizontal::Right))
                .align_right(Length::Fill)
                .align_bottom(Length::Fill)
        ]
        .into()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        pub use Message::*;

        match message {
            ConfigLoaded(c) => {
                self.config = c.clone();

                // Nothing is using it here as it's still in the initialization process so it's ok to use blocking_write
                let mut config = self.server_config.blocking_write();
                *config = c;

                return Task::batch([
                    Task::perform(Repositories::initialize(self.server_config.clone()), |r| {
                        RepositoryLoaded(r)
                    }),
                    Task::future(async move {
                        let mut settings_pack = SettingsPack::new();
                        settings_pack.set_alert_mask(
                            AlertCategory::Error | AlertCategory::Storage | AlertCategory::Status,
                        );

                        let client =
                            TorrentClient::create(AnawtOptions::new().settings_pack(settings_pack));

                        // TODO: this should not kill the client
                        match client.load("./data/torrents".into()).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!("Failed to load torrents: {}", e);
                                return PostToast(Toast {
                                    title: "Failed to load torrents".to_string(),
                                    body: e.to_string(),
                                    ty: ToastType::Error,
                                });
                            }
                        }

                        TorrentClientLoaded(client)
                    }),
                ]);
            }
            RepositoryLoaded(r) => {
                self.repositories = Some(r.clone());

                let server_config = self.server_config.clone();
                let repositories = r.clone();
                tokio::spawn(async move {
                    let server = AuroraServer::new();
                    server
                        .run(server_config.clone(), repositories)
                        .await
                        .unwrap();
                });

                let config = self.config.clone();

                return Task::perform(AuroraClient::new(r, config), |c| ClientLoaded(c));
            }
            TorrentClientLoaded(t) => {
                self.torrent_client = Some(t);
            }
            ClientLoaded(aurora_client) => {
                self.client = Some(aurora_client);
            }
            DownloadTorrent { magnet, path } => {
                if let Some(torrent_client) = &self.torrent_client {
                    let client = torrent_client.clone();

                    return Task::future(async move {
                        let _ = client.add_magnet(&magnet, &path).await;
                        Message::Nothing
                    });
                }
            }
            ChangeView(v) => {
                let old_view = std::mem::replace(&mut self.view, v);
                self.history.push(old_view);
                return View::on_enter(self);
            }
            ViewMessage(m) => {
                return View::update(m, self);
            }
            ModalMessage(m) => {
                return Modal::update(m, self);
            }
            BackHistory => {
                if let Some(v) = self.history.pop() {
                    self.view = v;
                    return View::on_enter(self);
                }
            }
            ToastSenderReady(tx) => {
                if self.toast_tx.is_some() {
                    error!("Tried to set ToastSenderReady twice");
                } else {
                    self.toast_tx = Some(tx);
                }
            }
            PostToast(toast) => {
                self.add_toast(toast);
            }
            CloseToast(i) => {
                self.toasts.remove(i);
            }
            OpenModal(m) => {
                self.modal = Some(m);
            }
            CloseModal => {
                self.close_modal();
            }
            SaveTorrent => {
                if let Some(client) = &self.torrent_client {
                    let client = client.clone();
                    return Task::future(async move {
                        client.save(PathBuf::from("./data/torrents")).await.unwrap();
                        Message::Nothing
                    });
                }
            }
            OpenWindow => {
                return window::open(window::Settings {
                    size: iced::Size::new(800.0, 600.0),
                    ..Default::default()
                })
                .1
                .map(|_| Message::Nothing);
            }
            Exchange => {
                // if self.exchanging {
                //     return Task::none();
                // }

                // let Some(mut client) = self.client.clone() else {
                //     return Task::none();
                // };

                // let repository = match &self.repositories {
                //     Some(r) => r.clone(),
                //     None => return Task::none(),
                // };

                // self.exchanging = true;

                // let self_key = self.config.public_key().clone();

                // return Task::future(async move {
                //     let Ok(user) = repository.user().await.get_random_user().await else {
                //         error!("Failed to get random user");
                //         return Message::FinishExchange;
                //     };

                //     if user.pub_key() == &self_key {
                //         //TODO: remove this later and move duty to get_random_user
                //         error!("Cannot exchange with self");
                //         return Message::FinishExchange;
                //     }

                //     info!("Exchanging with {}", user.address());
                //     match client.routine_exchange(user.address()).await {
                //         Ok(()) => {}
                //         Err(e) => {
                //             error!("Failed to exchange: {}", e);
                //         }
                //     }

                //     Message::FinishExchange
                // });
            }
            FinishExchange => {
                self.exchanging = false;
            }
            Nothing => {}
        }

        Task::none()
    }

    pub fn add_toast(&mut self, toast: Toast) {
        self.toasts.push(toast);
    }

    pub fn close_modal(&mut self) {
        self.modal = None;
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
        let toast_subscription = Subscription::run(toast_worker);
        let view_subscription = self.view.subscription();

        Subscription::batch([
            iced::time::every(std::time::Duration::from_millis(500)).map(|_| Message::Nothing),
            //iced::time::every(std::time::Duration::from_millis(5000)).map(|_| Message::Exchange),
            toast_subscription,
            view_subscription,
        ])
    }
}
