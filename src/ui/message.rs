use super::WindowType;
use crate::config::AkarekoConfig;
use crate::db::FullSyncTarget;
use crate::db::Repositories;
use crate::db::schedule::Schedule;
use crate::db::user::I2PAddress;
use crate::server::client::AkarekoClient;
use crate::ui::TrayIconMessage;
use crate::ui::components::modal::Modal;
use crate::ui::components::modal::ModalMessage;
use crate::ui::components::toast::Toast;
use crate::ui::views::View;
use crate::ui::views::ViewMessage;
use anawt::TorrentClient;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum Message {
    OpenWindow(WindowType),
    CloseWindow(iced::window::Id),
    Exit,

    RepositoryLoaded(Repositories),
    ConfigLoaded(AkarekoConfig),
    TorrentClientLoaded(TorrentClient),
    ClientLoaded(AkarekoClient),
    DownloadTorrent { magnet: String, path: String },
    ChangeView(View),
    ViewMessage(ViewMessage),
    BackHistory,

    ToastSenderReady(mpsc::Sender<Toast>),
    PostToast(Toast),
    CloseToast(usize),

    ModalMessage(ModalMessage),
    OpenModal(Modal),
    CloseModal,

    SaveTorrent,

    AddSchedule(Schedule),
    RemoveSchedule(Schedule),
    LoadFullSyncAddresses(Vec<(I2PAddress, FullSyncTarget)>),
    TryConsumeSchedule,

    TrayIconMessage(TrayIconMessage),

    Nothing,
}
