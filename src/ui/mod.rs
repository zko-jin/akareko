use anawt::TorrentClient;
use freya::{
    prelude::*,
    radio::{RadioChannel, RadioStation, use_share_radio},
};

use crate::{
    config::AkarekoConfig,
    db::{
        Repositories,
        index::{Index, tags::IndexTag},
    },
    server::client::pool::ClientPool,
    ui::{
        components::{layout_button, no_reaction_button},
        icons::ARROW_LEFT_ICON,
        router::RouteComponent,
    },
};

pub mod app_manager;
mod components;
mod icons;
mod queries;
mod router;
mod theme;
pub use router::{Route, RouteContext};

const DEFAULT_PAGE_PADDING: Gaps = Gaps::new(20., 50., 0., 50.);
const DEFAULT_CORNER_RADIUS: f32 = 10.;
const UNKNOWN_COVER: (&'static str, Bytes) = (
    "unknown_cover",
    Bytes::from_static(include_bytes!("../../assets/placeholder_cover.png")),
);

#[derive(Clone)]
struct IndexComponent<I: IndexTag + 'static> {
    index: Index<I>,
}
impl<'a, I: IndexTag> PartialEq for IndexComponent<I> {
    fn eq(&self, other: &Self) -> bool {
        self.index.hash() == other.index.hash()
    }
}

impl<I: IndexTag + 'static> Component for IndexComponent<I> {
    fn render(&self) -> impl IntoElement {
        let i = self.index.clone();
        let on_press = move |_| {
            RouteContext::get().push(Route::Manga {
                index: i.clone().transmute(),
            });
        };

        let cover_image = no_reaction_button()
            .child(
                ImageViewer::new(UNKNOWN_COVER)
                    .corner_radius(DEFAULT_CORNER_RADIUS)
                    .height(Size::px(200.)),
            )
            .on_press(on_press.clone());

        rect()
            .horizontal()
            .spacing(10.)
            .border(Some(Border::new().fill(Color::GRAY).width(2.)))
            .padding(10.)
            .with_corner_radius(DEFAULT_CORNER_RADIUS)
            .child(cover_image)
            .child(
                rect().width(Size::px(250.)).child(
                    no_reaction_button()
                        .child(
                            label()
                                .text(self.index.title().clone())
                                .font_weight(FontWeight::BOLD),
                        )
                        .on_press(on_press),
                ),
            )
    }
}

#[derive(Clone, Copy)]
pub struct AkarekoApp {
    radio_station: RadioStation<AppState, AppChannel>,
    router: RouteContext,
}

impl AkarekoApp {
    pub fn new(radio_station: RadioStation<AppState, AppChannel>, router: RouteContext) -> Self {
        AkarekoApp {
            radio_station,
            router,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
pub enum AppChannel {
    Status,
    Config,
    Repository,
    Server,
    Client,
    TorrentClient,

    Window,
}

pub enum ResourceState<T, E> {
    Pending,
    Error(E),
    Loading,
    Loaded(T),
}

impl<T, E> ResourceState<T, E> {
    pub fn unwrap_ref(&self) -> &T {
        match self {
            ResourceState::Pending => panic!("ResourceState::Pending"),
            ResourceState::Error(_) => panic!("ResourceState::Error"),
            ResourceState::Loading => panic!("ResourceState::Loading"),
            ResourceState::Loaded(t) => t,
        }
    }

    pub fn mut_unwrap_ref(&mut self) -> &mut T {
        match self {
            ResourceState::Pending => panic!("ResourceState::Pending"),
            ResourceState::Error(_) => panic!("ResourceState::Error"),
            ResourceState::Loading => panic!("ResourceState::Loading"),
            ResourceState::Loaded(t) => t,
        }
    }
}

impl<T: Clone, E: Clone> Clone for ResourceState<T, E> {
    fn clone(&self) -> Self {
        match self {
            ResourceState::Pending => ResourceState::Pending,
            ResourceState::Error(e) => ResourceState::Error(e.clone()),
            ResourceState::Loading => ResourceState::Loading,
            ResourceState::Loaded(t) => ResourceState::Loaded(t.clone()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppWindowType {
    Main,
}

pub struct AppState {
    pub config: ResourceState<AkarekoConfig, ()>,
    pub repositories: ResourceState<Repositories, ()>,
    pub torrent_client: ResourceState<TorrentClient, ()>,
    pub server: ResourceState<(), ()>,
    pub client: ResourceState<ClientPool, ()>,
    pub windows_state: AppWindowState,
}

pub struct AppWindowState {
    windows: Vec<AppWindowType>,
}

impl AppWindowState {
    fn new() -> Self {
        Self { windows: vec![] }
    }

    pub fn try_add_window(&mut self, window_type: AppWindowType) -> bool {
        match window_type {
            AppWindowType::Main => {
                if !self.windows.contains(&window_type) {
                    self.windows.push(AppWindowType::Main);
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn remove_main_window(&mut self) {
        self.windows.retain(|&w| w != AppWindowType::Main);
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: ResourceState::Pending,
            repositories: ResourceState::Pending,
            torrent_client: ResourceState::Pending,
            server: ResourceState::Pending,
            client: ResourceState::Pending,
            windows_state: AppWindowState::new(),
        }
    }
}

impl RadioChannel<AppState> for AppChannel {
    fn derive_channel(self, _radio: &AppState) -> Vec<Self> {
        match self {
            AppChannel::TorrentClient
            | AppChannel::Config
            | AppChannel::Repository
            | AppChannel::Server
            | AppChannel::Client => vec![self, AppChannel::Status],
            _ => vec![self],
        }
    }
}

impl App for AkarekoApp {
    fn render(&self) -> impl IntoElement {
        use_share_radio(move || self.radio_station);
        use_provide_context(|| self.router);
        // use_provide_context(|| self.radio_station);
        use_hook(|| {
            let ctx = self.radio_station;
            provide_context_for_scope_id(ctx.clone(), ScopeId::ROOT);
            ctx
        });
        use_init_theme(|| dark_theme());
        Layout
    }
}

#[derive(PartialEq)]
struct Layout;
impl Component for Layout {
    fn render(&self) -> impl IntoElement {
        rect()
            .horizontal()
            .expanded()
            .child(
                rect()
                    .vertical()
                    .width(Size::px(200.))
                    .height(Size::Fill)
                    .child(
                        rect().horizontal().child(
                            Button::new()
                                .child(svg(ARROW_LEFT_ICON))
                                .enabled(RouteContext::get().can_go_back())
                                .on_press(|_| {
                                    RouteContext::get().go_back();
                                }),
                        ),
                    )
                    .child(layout_button(Route::Home))
                    .child(layout_button(Route::MangaList))
                    .child(layout_button(Route::Settings)),
            )
            .child(
                rect()
                    .child(RouteComponent)
                    .expanded()
                    .margin((5.0, 5.0, 5.0, 0.0))
                    .overflow(Overflow::Clip)
                    .corner_radius(DEFAULT_CORNER_RADIUS)
                    .background(Color::WHITE),
            )
            .background(Color::GRAY)
    }
}

// use anawt::{AlertCategory, SettingsPack, TorrentClient,
// options::AnawtOptions}; use clap::Parser;
// use iced::{
//     Length, Subscription, Task, Theme, alignment,
//     widget::{Column, Container, column, stack, text},
//     window,
// };
// use rclite::Arc;
// use std::{collections::BTreeMap, path::PathBuf};
// use tokio::sync::{RwLock, mpsc};
// use tracing::{error, info};
// use trayicon::TrayIcon;

// use crate::{
//     CliArgs,
//     config::AkarekoConfig,
//     db::{
//         FullSyncTarget, Repositories,
//         index::tags::MangaTag,
//         schedule::{Schedule, ScheduleType, Scheduler},
//         user::I2PAddress,
//     },
//     helpers::LiFo,
//     server::{
//         AkarekoServer,
//         client::{AkarekoClient, TIME_OFFSET, pool::ClientPool},
//     },
//     types::Timestamp,
//     ui::{
//         components::{
//             modal::{Modal, ModalMessage, modal},
//             toast::{Toast, ToastType, toast_worker},
//         },
//         tray::initialize_tray_icon,
//         views::{View, ViewMessage, home::HomeView},
//     },
// };

// mod components;
// mod icons;
// mod message;
// mod style;
// mod tray;
// mod views;

// #[derive(Debug, Clone, PartialEq)]
// pub enum TrayIconMessage {
//     OpenWindow,
//     Exit,
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum WindowType {
//     Main,
// }

// pub struct AppState {
//     repositories: Option<Repositories>,
//     config: AkarekoConfig,
//     server_config: Arc<RwLock<AkarekoConfig>>,

//     view: View,
//     history: LiFo<View, 10>,

//     scheduler: Scheduler,

//     client_pool: Option<ClientPool>,
//     torrent_client: Option<TorrentClient>,

//     toast_tx: Option<mpsc::Sender<Toast>>,
//     toasts: Vec<Toast>,

//     theme: Theme,

//     tray_icon: TrayIcon<TrayIconMessage>,

//     windows: BTreeMap<window::Id, WindowType>,

//     modal: Option<Modal>,
// }

// impl AppState {
//     pub fn new() -> Self {
//         let tray_icon = initialize_tray_icon();
//         Self {
//             repositories: None,
//             config: AkarekoConfig::default(),
//             client_pool: None,
//             server_config: Arc::new(RwLock::new(AkarekoConfig::default())),
//             torrent_client: None,
//             view: View::Home(HomeView::new()),
//             history: LiFo::new(),
//             toast_tx: None,
//             toasts: Vec::new(),
//             theme: Theme::CatppuccinMocha,
//             modal: None,
//             scheduler: Scheduler::new(),
//             tray_icon,
//             windows: BTreeMap::new(),
//         }
//     }

//     pub fn boot() -> (AppState, Task<message::Message>) {
//         let args = CliArgs::parse();

//         let open_task = match args.minimized {
//             true => Task::none(),
//             false =>
// Task::done(message::Message::OpenWindow(WindowType::Main)),         };
//         (
//             AppState::new(),
//             open_task.chain(Task::perform(AkarekoConfig::load(), |c| {
//                 message::Message::ConfigLoaded(c)
//             })),
//         )
//     }

//     fn has_initialized(&self) -> bool {
//         self.repositories.is_some() && self.client_pool.is_some() &&
// self.torrent_client.is_some()     }

//     pub fn theme(&self) -> Theme {
//         self.theme.clone()
//     }

//     pub fn view(&self, _id: window::Id) -> iced::Element<'_,
// message::Message> {         if !self.has_initialized() {
//             return column![text("Loading...")].into();
//         }

//         let sidebar = components::sidebar::sidebar(self.history.can_pop());

//         let base = column![sidebar, View::view(self)]
//             .width(iced::Length::Fill)
//             .height(iced::Length::Fill);

//         let base = if self.modal.is_some() {
//             modal(base, Modal::view(self), message::Message::CloseModal)
//         } else {
//             base.into()
//         };

//         let toasts = self
//             .toasts
//             .iter()
//             .rev()
//             .enumerate()
//             .map(|(i, t)| t.view(i))
//             .collect();

//         stack![
//             base,
//
// Container::new(Column::from_vec(toasts).
// align_x(alignment::Horizontal::Right))
// .align_right(Length::Fill)                 .align_bottom(Length::Fill)
//         ]
//         .into()
//     }

//     pub fn update(&mut self, message: message::Message) ->
// Task<message::Message> {         match message {
//             message::Message::Exit => return iced::exit(),
//             message::Message::ConfigLoaded(c) => {
//                 self.config = c.clone();

//                 // Nothing is using it here as it's still in the
// initialization process so it's                 // ok to use blocking_write
//                 {
//                     let mut server_config =
// self.server_config.blocking_write();                     *server_config = c;
//                 }

//                 let config = self.config.clone();

//                 return Task::batch([
//                     Task::perform(AkarekoClient::new(config.clone()), |c| {
//                         message::Message::ClientLoaded(c)
//                     }),
//                     Task::future(async move {
//                         info!("Initializing Repositories...");
//                         let r = Repositories::initialize(&config).await;
//                         message::Message::RepositoryLoaded(r)
//                     }),
//                     Task::future(async move {
//                         let mut settings_pack = SettingsPack::new();
//                         settings_pack.set_alert_mask(
//                             AlertCategory::Error | AlertCategory::Storage |
// AlertCategory::Status,                         );

//                         let client =
//
// TorrentClient::create(AnawtOptions::new().settings_pack(settings_pack));

//                         // TODO: this should not kill the client
//                         match client.load("./data/torrents".into()).await {
//                             Ok(_) => {}
//                             Err(e) => {
//                                 error!("Failed to load torrents: {}", e);
//                                 return message::Message::PostToast(Toast {
//                                     title: "Failed to load
// torrents".to_string(),                                     body:
// e.to_string(),                                     ty: ToastType::Error,
//                                 });
//                             }
//                         }

//                         message::Message::TorrentClientLoaded(client)
//                     }),
//                 ]);
//             }
//             message::Message::RepositoryLoaded(r) => {
//                 self.repositories = Some(r.clone());

//                 let server_config = self.server_config.clone();
//                 let server_repo = r.clone();
//                 tokio::spawn(async move {
//                     let server = AkarekoServer::new();
//                     server
//                         .run(server_config.clone(), server_repo)
//                         .await
//                         .unwrap();
//                 });
//                 return Task::future(async move {
//                     let targets = r.full_sync_addresses().await.unwrap();
//                     let pub_keys = targets
//                         .iter()
//                         .map(|t| t.pub_key.clone())
//                         .collect::<Vec<_>>();

//                     let users = r.user().get_users(pub_keys).await.unwrap();

//                     let addresses: Vec<(I2PAddress, FullSyncTarget)> = users
//                         .into_iter()
//                         .zip(targets)
//                         .map(|(u, t)| (u.into_address(), t))
//                         .collect();

//                     message::Message::LoadFullSyncAddresses(addresses)
//                 });
//             }
//             message::Message::TorrentClientLoaded(t) => {
//                 self.torrent_client = Some(t);
//             }
//             message::Message::ClientLoaded(client) => {
//                 self.client_pool = Some(ClientPool::new(
//                     client,
//                     self.config.max_client_connections() as u16,
//                 ));
//             }
//             message::Message::DownloadTorrent { magnet, path } => {
//                 if let Some(torrent_client) = &self.torrent_client {
//                     let client = torrent_client.clone();

//                     return Task::future(async move {
//                         let _ = client.add_magnet(&magnet, &path).await;
//                         message::Message::Nothing
//                     });
//                 }
//             }
//             message::Message::ChangeView(v) => {
//                 let old_view = std::mem::replace(&mut self.view, v);
//                 self.history.push(old_view);
//                 return View::on_enter(self);
//             }
//             message::Message::ViewMessage(m) => {
//                 return View::update(m, self);
//             }
//             message::Message::ModalMessage(m) => {
//                 return Modal::update(m, self);
//             }
//             message::Message::BackHistory => {
//                 if let Some(v) = self.history.pop() {
//                     self.view = v;
//                     return View::on_enter(self);
//                 }
//             }
//             message::Message::ToastSenderReady(tx) => {
//                 if self.toast_tx.is_some() {
//                     error!("Tried to set ToastSenderReady twice");
//                 } else {
//                     self.toast_tx = Some(tx);
//                 }
//             }
//             message::Message::PostToast(toast) => {
//                 self.add_toast(toast);
//             }
//             message::Message::CloseToast(i) => {
//                 self.toasts.remove(i);
//             }
//             message::Message::OpenModal(m) => {
//                 self.modal = Some(m);
//             }
//             message::Message::CloseModal => {
//                 self.close_modal();
//             }
//             message::Message::SaveTorrent => {
//                 if let Some(client) = &self.torrent_client {
//                     let client = client.clone();
//                     return Task::future(async move {
//
// client.save(PathBuf::from("./data/torrents")).await.unwrap();
// message::Message::Nothing                     });
//                 }
//             }
//             message::Message::OpenWindow(window_type) => {
//                 match window_type {
//                     WindowType::Main => {
//                         if self.windows.values().any(|v| *v == window_type) {
//                             return Task::done(message::Message::Nothing);
//                         }
//                     }
//                 }

//                 let (id, task) = window::open(window::Settings {
//                     size: iced::Size::new(800.0, 600.0),
//                     maximized: true,
//                     exit_on_close_request: false,
//                     ..Default::default()
//                 });

//                 self.windows.insert(id, window_type);

//                 return task.map(|_| message::Message::Nothing);
//             }
//             message::Message::CloseWindow(id) => {
//                 let window_type = self.windows.remove(&id).unwrap();
//                 return window::close(id);
//             }
//             message::Message::AddSchedule(schedule) => {
//                 self.scheduler.schedule(schedule);
//             }
//             message::Message::RemoveSchedule(schedule) => {
//                 self.scheduler.remove(schedule);
//             }
//             message::Message::TryConsumeSchedule => {
//                 let (Some(pool), Some(db)) = (self.client_pool.clone(),
// self.repositories.clone())                 else {
//                     return Task::none();
//                 };
//                 let Some(schedule) = self.scheduler.try_next() else {
//                     return Task::none();
//                 };

//                 info!("Consuming schedule: {schedule:?}");

//                 let scheduler_config =
// self.config.scheduler_config().clone();                 return
// Task::future(async move {                     let mut client =
// pool.get_client().await;                     let (server_timestamp,
// increment) = match schedule.schedule_type {
// ScheduleType::FullSync(ref pub_key) => {                             let
// server_timestamp = match client
// .sync_events(&schedule.address, schedule.last_sync, &db)
// .await                             {
//                                 Ok(t) => t,
//                                 Err(e) => {
//                                     error!("Failed to sync events: {}", e);
//                                     return
// message::Message::AddSchedule(Schedule {
// when: Timestamp::now()
//                                             + scheduler_config.
//                                               full_sync_interval,
//                                         address: schedule.address,
//                                         schedule_type:
// schedule.schedule_type,                                         last_sync:
// schedule.last_sync,                                     });
//                                 }
//                             };

//                             db.upsert_full_sync_address(FullSyncTarget {
//                                 pub_key: pub_key.clone(),
//                                 last_sync: server_timestamp,
//                             })
//                             .await
//                             .unwrap();

//                             (server_timestamp,
// scheduler_config.full_sync_interval)                         }
//                         ScheduleType::SyncMangaContent(ref hash) => {
//                             let filter = db
//                                 .index()
//                                 .make_filter::<MangaTag>(
//                                     &hash,
//                                     Some(schedule.last_sync - TIME_OFFSET),
//                                 )
//                                 .await
//                                 .unwrap();

//                             client
//                                 .get_manga_content(
//                                     &schedule.address,
//                                     db.index(),
//                                     hash.clone(),
//                                     Some(schedule.last_sync),
//                                     Some(filter),
//                                 )
//                                 .await
//                                 .unwrap();

//                             (Timestamp::new(0), Timestamp::new(0))
//                         }
//                         ScheduleType::SyncPost(ref topic) => {
//                             let filter = db
//                                 .make_posts_filter(
//                                     topic.clone(),
//                                     Some(schedule.last_sync - TIME_OFFSET),
//                                 )
//                                 .await
//                                 .unwrap();

//                             (Timestamp::new(0), Timestamp::new(0))
//                         }
//                     };

//                     message::Message::AddSchedule(Schedule {
//                         when: Timestamp::now() + increment,
//                         address: schedule.address,
//                         schedule_type: schedule.schedule_type,
//                         last_sync: server_timestamp,
//                     })
//                 });
//             }
//             message::Message::LoadFullSyncAddresses(a) => {
//                 for (address, target) in a {
//                     self.scheduler.schedule(Schedule {
//                         when: target.last_sync +
// self.config.scheduler_config().full_sync_interval,
// last_sync: target.last_sync,                         address,
//                         schedule_type:
// ScheduleType::FullSync(target.pub_key),                     });
//                 }
//             }
//             message::Message::TrayIconMessage(m) => match m {
//                 TrayIconMessage::OpenWindow => {
//                     return
// Task::done(message::Message::OpenWindow(WindowType::Main));                 }
//                 TrayIconMessage::Exit => {
//                     return Task::done(message::Message::Exit);
//                 }
//             },
//             message::Message::Nothing => {}
//         }

//         Task::none()
//     }

//     pub fn add_toast(&mut self, toast: Toast) {
//         self.toasts.push(toast);
//     }

//     pub fn close_modal(&mut self) {
//         self.modal = None;
//     }

//     pub fn subscription(&self) -> iced::Subscription<message::Message> {
//         let toast_subscription = Subscription::run(toast_worker);
//         let view_subscription = self.view.subscription();

//         let tray_icon_subscription = self.tray_icon.subscribe();

//         Subscription::batch([
//             iced::time::every(std::time::Duration::from_millis(500))
//                 .map(|_| message::Message::Nothing),
//             iced::time::every(std::time::Duration::from_millis(3500))
//                 .map(|_| message::Message::TryConsumeSchedule),
//             toast_subscription,
//             view_subscription,
//             window::close_requests().map(message::Message::CloseWindow),
//             tray_icon_subscription.map(|m|
// message::Message::TrayIconMessage(m)),         ])
//     }
// }
