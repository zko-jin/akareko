use anawt::{TorrentClient, options::AnawtOptions};
use emissary_core::{Config, Ntcp2Config, SamConfig, Ssu2Config, TransitConfig, router::Router};
use emissary_util::{
    reseeder::Reseeder,
    runtime::tokio::Runtime,
    storage::{Storage, StorageBundle},
};
use freya::radio::RadioStation;
use rclite::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};
use yosemite::{RouterApi, Session, style};

use crate::{
    config::{AkarekoConfig, I2PRouterConfig},
    daemon::resource_state::{AppResources, AppResourcesManager, ResourceStateManager},
    db::{
        FullSyncTarget, Repositories,
        index::tags::MangaTag,
        schedule::{Schedule, ScheduleType, Scheduler},
    },
    helpers::b32_from_pub_b64,
    server::{
        AkarekoServer,
        client::{AkarekoClient, TIME_OFFSET, pool::ClientPool},
    },
    types::Timestamp,
    ui::{AppChannel, AppState},
};

pub mod resource_state;

pub enum Event {
    RemoveMainWindow,
    AddSchedule(Schedule),
}

pub struct AppManager {
    resources: AppResourcesManager,
    state: RadioStation<AppState, AppChannel>,

    scheduler: Scheduler,

    sam_session: ResourceStateManager<Arc<Mutex<Session<style::Primary>>>, ()>,
    tx: tokio::sync::mpsc::UnboundedSender<Event>,
    rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
}

pub async fn init_router(sam_tcp_port: u16, sam_udp_port: u16) -> Router<Runtime> {
    let storage = Storage::new::<Runtime>(None).await.unwrap();
    let StorageBundle {
        ntcp2_iv,
        ntcp2_key,
        profiles,
        router_info,
        mut routers,
        signing_key,
        static_key,
        ssu2_intro_key,
        ssu2_static_key,
    } = storage.load().await;

    if routers.is_empty() {
        match Reseeder::reseed::<Runtime>(None, false).await {
            Ok(reseed_routers) => {
                for info in reseed_routers {
                    let _ = storage
                        .store_router_info(info.name.to_string(), info.router_info.clone())
                        .await;
                    routers.push(info.router_info);
                }
            }

            Err(error) => tracing::error!(
                num_routers = routers.len(),
                ?error,
                "failed to reseed router",
            ),
        }
    }

    let i2p_router_config = Config {
        // allow_local: true,
        // insecure_tunnels: true,
        ntcp2: Some(Ntcp2Config {
            port: 25515,
            key: ntcp2_key,
            iv: ntcp2_iv,
            publish: true,
            ipv4_host: None,
            ipv6_host: None,
            ipv4: true,
            ipv6: true,
            ml_kem: Some(4),
            disable_pq: false,
        }),
        ssu2: Some(Ssu2Config {
            intro_key: ssu2_intro_key,
            static_key: ssu2_static_key,
            ipv4: true,
            ipv4_host: None,
            ipv6: true,
            ipv6_host: None,
            port: 25515,
            publish: true,
            ipv4_mtu: None,
            ipv6_mtu: None,
            disable_pq: false,
            ml_kem: Some("4".to_string()),
        }),
        samv3_config: Some(SamConfig {
            tcp_port: sam_tcp_port,
            udp_port: sam_udp_port,
            host: "127.0.0.1".to_string(),
        }),
        routers,
        profiles,
        router_info,
        static_key: Some(static_key),
        signing_key: Some(signing_key),
        transit: Some(TransitConfig {
            max_tunnels: Some(1000),
        }),
        ..Config::default()
    };

    let (router, _events, router_info) = Router::<Runtime>::new(
        i2p_router_config,
        None,
        Some(std::sync::Arc::new(storage.clone())),
    )
    .await
    .map_err(|error| panic!("failed to start router: {error}"))
    .unwrap();

    storage.store_local_router_info(router_info).await.unwrap();

    router
}

impl AppManager {
    #[cfg(feature = "ui")]
    pub fn resources(&self) -> AppResources {
        self.resources.get_resources()
    }

    pub async fn run_manager(mut self) {
        self.load_i2p_router_config();
        self.load_config();
        self.start_torrent_client();

        self.process_events().await;
    }

    pub fn new(
        state: RadioStation<AppState, AppChannel>,
    ) -> (AppManager, tokio::sync::mpsc::UnboundedSender<Event>) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let manager = AppManager {
            resources: Default::default(),
            state,
            sam_session: Default::default(),
            scheduler: Scheduler::new(),
            tx: tx.clone(),
            rx,
        };

        (manager, tx)
    }

    pub fn load_config(&mut self) {
        self.resources
            .config
            .load(async { Ok(AkarekoConfig::load().await) });
    }

    pub fn load_i2p_router_config(&mut self) {
        self.resources
            .i2p_router_config
            .load(async { Ok(I2PRouterConfig::load().await) });
    }

    pub async fn start_router(&mut self) {
        let config = match self.resources.i2p_router_config.get_value() {
            Some(c) => c,
            _ => return,
        };

        let router = init_router(config.sam_tcp_port(), config.sam_udp_port()).await;

        tokio::spawn(router);
        tracing::info!("Initialized I2P router");
    }

    pub async fn start_sam_session(&mut self) {
        let i2p_config = match self.resources.i2p_router_config.get_value() {
            Some(c) => c,
            _ => return,
        };

        let config = match self.resources.config.get_value() {
            Some(mut c) => {
                if c.eepsite_key().is_empty() {
                    let (destination, private_key) = RouterApi::new(i2p_config.sam_tcp_port())
                        .generate_destination()
                        .await
                        .unwrap();
                    c.set_eepsite_data(b32_from_pub_b64(&destination).unwrap(), private_key);
                    self.resources.config.force_load(c.clone());
                    c.save().await.unwrap();
                }
                c
            }
            _ => return,
        };

        self.sam_session.load(async move {
            let sam_session = Session::<style::Primary>::new(yosemite::SessionOptions {
                nickname: "Akareko".to_string(),
                samv3_tcp_port: i2p_config.sam_tcp_port(),
                samv3_udp_port: i2p_config.sam_udp_port(),
                destination: yosemite::DestinationKind::Persistent {
                    private_key: config.eepsite_key().clone(),
                },
                ..Default::default()
            })
            .await
            .unwrap();

            tracing::info!("Loaded SAM session: {}", sam_session.destination());
            Ok(Arc::new(Mutex::new(sam_session)))
        });
    }

    pub fn start_torrent_client(&mut self) {
        self.resources.torrent_client.load(async {
            let torrent_client = TorrentClient::create(AnawtOptions::new());
            match torrent_client.load("./data/torrents".into()).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to load torrents: {}", e);
                }
            }
            Ok(torrent_client)
        });
    }

    pub fn start_client(&mut self) {
        let config = match self.resources.config.get_value() {
            Some(c) => c,
            _ => return,
        };

        let session = match self.sam_session.get_value() {
            Some(s) => s,
            _ => return,
        };

        self.resources.client.load(async move {
            let subsession = session
                .lock()
                .await
                .create_subsession::<style::Stream>(yosemite::SessionOptions {
                    nickname: "AkarekoClient".to_string(),
                    // samv3_tcp_port: config.sam_tcp_port(),
                    // samv3_udp_port: config.sam_udp_port(),
                    // destination: yosemite::DestinationKind::Persistent {
                    //     private_key: config.eepsite_key().clone(),
                    // },
                    ..Default::default()
                })
                .await
                .unwrap();

            tracing::info!("Loaded client SAM session: {}", subsession.destination());

            Ok(ClientPool::new(
                AkarekoClient::new(subsession, config.clone()).await,
                config.max_client_connections(),
            ))
        });
    }

    pub fn start_server(&mut self) {
        let config = match self.resources.config.get_value() {
            Some(c) => c,
            _ => return,
        };

        let repositories = match self.resources.repositories.get_value() {
            Some(r) => r,
            _ => return,
        };

        let session = match self.sam_session.get_value() {
            Some(s) => s,
            _ => return,
        };

        self.resources.server.load(async move {
            let subsession = session
                .lock()
                .await
                .create_subsession::<style::Stream>(yosemite::SessionOptions {
                    nickname: "AkarekoServer".to_string(),
                    // samv3_tcp_port: config.sam_tcp_port(),
                    // samv3_udp_port: config.sam_udp_port(),
                    // destination: yosemite::DestinationKind::Persistent {
                    //     private_key: config.eepsite_key().clone(),
                    // },
                    ..Default::default()
                })
                .await
                .unwrap();

            tracing::info!("Loaded server SAM session: {}", subsession.destination());

            let config = Arc::new(RwLock::new(config));

            tokio::spawn(async move {
                let server = AkarekoServer::new();
                server.run(config, repositories, subsession).await.unwrap();
            });

            Ok(())
        });
    }

    pub fn start_repository(&mut self) {
        let config = match self.resources.config.get_value() {
            Some(c) => c,
            _ => return,
        };

        self.resources
            .repositories
            .load(async move { Ok(Repositories::initialize(&config).await) });
    }

    pub fn sync(&mut self, schedule: Schedule) {
        let (Some(pool), Some(db)) = (
            self.resources.client.get_value(),
            self.resources.repositories.get_value(),
        ) else {
            self.scheduler.schedule(schedule);
            warn!("Could not sync, missing resources");
            return;
        };

        info!("Consuming schedule: {schedule:?}");

        let scheduler_config = if let Some(config) = self.resources.config.get_value() {
            config.scheduler_config().clone()
        } else {
            return;
        };

        let tx = self.tx.clone();

        tokio::spawn(async move {
            let mut client = pool.get_client().await;

            let (server_timestamp, increment) = match schedule.schedule_type {
                ScheduleType::FullSync(ref pub_key) => {
                    let server_timestamp = match client
                        .sync_events(&schedule.address, schedule.last_sync, &db)
                        .await
                    {
                        Ok(t) => t,
                        Err(e) => {
                            error!("Failed to sync events: {}", e);
                            tx.send(Event::AddSchedule(Schedule {
                                when: Timestamp::now() + scheduler_config.full_sync_interval,
                                address: schedule.address,
                                schedule_type: schedule.schedule_type.clone(),
                                last_sync: schedule.last_sync,
                            }))
                            .unwrap();
                            return;
                        }
                    };
                    db.upsert_full_sync_address(FullSyncTarget {
                        pub_key: pub_key.clone(),
                        last_sync: server_timestamp,
                    })
                    .await
                    .unwrap();

                    (server_timestamp, scheduler_config.full_sync_interval)
                }
                ScheduleType::SyncMangaContent(ref hash) => {
                    let filter = db
                        .index()
                        .make_filter::<MangaTag>(&hash, Some(schedule.last_sync - TIME_OFFSET))
                        .await
                        .unwrap();

                    client
                        .get_manga_content(
                            &schedule.address,
                            db.index(),
                            hash.clone(),
                            Some(schedule.last_sync),
                            Some(filter),
                        )
                        .await
                        .unwrap();

                    (Timestamp::new(0), Timestamp::new(0))
                }
                ScheduleType::SyncPost(ref topic) => {
                    let filter = db
                        .make_posts_filter(topic.clone(), Some(schedule.last_sync - TIME_OFFSET))
                        .await
                        .unwrap();

                    (Timestamp::new(0), Timestamp::new(0))
                }
            };

            tx.send(Event::AddSchedule(Schedule {
                when: Timestamp::now() + increment,
                address: schedule.address,
                schedule_type: schedule.schedule_type,
                last_sync: server_timestamp,
            }))
            .unwrap();
        });
    }

    pub async fn process_events(&mut self) {
        loop {
            tokio::select! {
                val = self.rx.recv() => {
                    match val.unwrap() {
                        Event::RemoveMainWindow => {
                            self.state.write_channel(AppChannel::Window).window_state.remove_main_window();
                        },
                        Event::AddSchedule(schedule) => {
                            self.scheduler.schedule(schedule);
                        }
                    }
                }

                schedule = &mut self.scheduler => {
                    self.sync(schedule);
                }

                _ = &mut self.resources.i2p_router_config => {
                    self.start_router().await;
                }

                _ = &mut self.resources.config => {
                    self.start_client();
                    self.start_sam_session().await;
                    self.start_server();
                    self.start_repository();
                }

                _ = &mut self.sam_session => {
                    self.start_client();
                    self.start_server();
                }

                _ = &mut self.resources.repositories => {
                    self.start_server();
                }
                _ = &mut self.resources.torrent_client => {}
                _ = &mut self.resources.server => {}
                _ = &mut self.resources.client => {}
            }
        }
    }
}
