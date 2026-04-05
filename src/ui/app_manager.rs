use anawt::{TorrentClient, options::AnawtOptions};
use freya::radio::RadioStation;
use tokio::sync::RwLock;
use tracing::error;

use crate::{
    config::AkarekoConfig,
    db::Repositories,
    server::{
        AkarekoServer,
        client::{AkarekoClient, pool::ClientPool},
    },
    ui::{AppChannel, AppState, ResourceState},
};

pub enum Event {
    ReloadConfig,
}

enum LoadEvent {
    LoadedClient(ClientPool),
}

pub struct AppManager {
    client_thread: Option<tokio::task::JoinHandle<()>>,
    radio_station: RadioStation<AppState, AppChannel>,
    load_tx: tokio::sync::mpsc::UnboundedSender<LoadEvent>,
    load_rx: tokio::sync::mpsc::UnboundedReceiver<LoadEvent>,
    rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
}

impl AppManager {
    pub async fn run_manager(mut self) {
        self.radio_station.write_channel(AppChannel::Config).config = ResourceState::Loading;
        let config = AkarekoConfig::load().await;
        self.radio_station.write_channel(AppChannel::Config).config =
            ResourceState::Loaded(config.clone());

        self.radio_station
            .write_channel(AppChannel::TorrentClient)
            .torrent_client = ResourceState::Loading;
        let torrent_client = TorrentClient::create(AnawtOptions::new());
        match torrent_client.load("./data/torrents".into()).await {
            Ok(_) => {}
            Err(e) => {
                error!("Failed to load torrents: {}", e);
            }
        }
        self.radio_station
            .write_channel(AppChannel::TorrentClient)
            .torrent_client = ResourceState::Loaded(torrent_client);

        self.radio_station
            .write_channel(AppChannel::Repository)
            .repositories = ResourceState::Loading;
        let repos = Repositories::initialize(&config).await;
        self.radio_station
            .write_channel(AppChannel::Repository)
            .repositories = ResourceState::Loaded(repos.clone());

        self.radio_station.write_channel(AppChannel::Server).server = ResourceState::Loading;
        let server = AkarekoServer::new();
        let server_conf = rclite::Arc::new(RwLock::new(config.clone()));
        tokio::spawn(async move {
            server.run(server_conf, repos).await.unwrap();
        });
        self.radio_station.write_channel(AppChannel::Server).server = ResourceState::Loaded(());

        self.start_client_thread();

        self.process_events().await;
    }

    pub fn new(
        radio_station: RadioStation<AppState, AppChannel>,
    ) -> (AppManager, tokio::sync::mpsc::UnboundedSender<Event>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let (load_tx, load_rx) = tokio::sync::mpsc::unbounded_channel();

        let mut manager = AppManager {
            client_thread: None,
            radio_station,
            load_tx,
            load_rx,
            rx,
        };

        manager.start_client_thread();

        (manager, tx)
    }

    pub fn start_client_thread(&mut self) {
        if let Some(t) = self.client_thread.take() {
            t.abort();
        };

        let config = match self.radio_station.read().config {
            ResourceState::Loaded(ref config) => config.clone(),
            _ => return,
        };

        self.radio_station.write_channel(AppChannel::Client).client = ResourceState::Loading;

        let mut load_tx = self.load_tx.clone();
        self.client_thread = Some(tokio::spawn(async move {
            let client = ClientPool::new(
                AkarekoClient::new(config.clone()).await,
                config.max_client_connections() as u16,
            );

            load_tx.send(LoadEvent::LoadedClient(client)).unwrap();
        }));
    }

    pub async fn process_events(&mut self) {
        loop {
            tokio::select! {
                val = self.rx.recv() => {
                    match val.unwrap() {
                        Event::ReloadConfig => todo!(),
                    }
                }
                val = self.load_rx.recv() => {
                    match val.unwrap() {
                        LoadEvent::LoadedClient(client) => {
                            self.radio_station.write_channel(AppChannel::Client).client =
                                ResourceState::Loaded(client);
                            self.client_thread = None;
                        }
                    }
                }
            }
        }
    }
}
