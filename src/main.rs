#![allow(dead_code)]

use std::path::PathBuf;

use anawt::TorrentClient;
use anawt::options::AnawtOptions;
use clap::Parser;
use freya::prelude::*;
use freya::radio::RadioStation;
use freya::tray::Icon;
use freya::tray::TrayEvent;
use freya::tray::TrayIconBuilder;
use freya::tray::menu::Menu;
use freya::tray::menu::MenuEvent;
use freya::tray::menu::MenuItem;
use futures::SinkExt;
use futures::channel::mpsc;
use futures::executor::block_on;
use rclite::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::error;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::AkarekoConfig;
use crate::db::Repositories;
use crate::server::AkarekoServer;
use crate::server::client::AkarekoClient;
use crate::server::client::pool::ClientPool;
use crate::ui::AkarekoApp;
use crate::ui::AppChannel;
use crate::ui::AppState;
use crate::ui::RouteContext;
use crate::ui::app_manager::AppManager;

// use crate::ui::AppState;

mod config;
mod db;
mod errors;
mod helpers;
mod server;
mod types;
mod ui;

#[derive(Parser)]
#[command(author, version, about)]
struct CliArgs {
    #[arg(long)]
    minimized: bool,
}

fn main() -> Result<(), ()> {
    // ==================== Tracing ====================
    let format = time::format_description::parse(":[minute]:[second]").expect("Cataplum");

    let timer = fmt::time::LocalTime::new(format);
    let filter = EnvFilter::builder().parse_lossy("none,akareko=trace,anawt=info");

    let stdout_log = fmt::layer()
        .compact()
        .with_line_number(false)
        .with_target(false)
        .with_timer(timer)
        .with_filter(filter);

    tracing_subscriber::registry().with(stdout_log).init();

    // ==================== End Tracing ====================

    info!("Initializing Application...");

    // iced::daemon(AppState::boot, AppState::update, AppState::view)
    //     .subscription(AppState::subscription)
    //     .theme(|s: &AppState, _| s.theme())
    //     .run()
    //     .unwrap();
    //
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    // Enter the Tokio context so its APIs (channels, timers, etc.) work.
    let _rt = rt.enter();

    let tray_icon = || {
        const ICON: &'static [u8] = include_bytes!("../assets/tray_icon.ico");
        let tray_menu = Menu::new();
        let _ = tray_menu.append(&MenuItem::with_id("open", "Open", true, None));
        let _ = tray_menu.append(&MenuItem::with_id("quit", "Quit", true, None));

        let (icon, width, height) = {
            let image = image::load_from_memory(ICON).unwrap().into_rgba8();
            (image.to_vec(), image.width(), image.height())
        };

        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("Akareko")
            .with_icon(Icon::from_rgba(icon, width, height).unwrap())
            .build()
            .unwrap()
    };

    let mut radio_station = RadioStation::<AppState, AppChannel>::create_global(AppState::new());
    // let router = RouterContext::create_global::<ui::Route>(
    //     RouterConfig::default().with_initial_path(Route::Home),
    // );
    let router = RouteContext::create_global();

    let tray_handler = move |ev, mut ctx: RendererContext| match ev {
        TrayEvent::Menu(MenuEvent { id }) if id == "open" => {
            // ctx.launch_window(WindowConfig::new(app).with_size(500., 450.));
        }
        TrayEvent::Menu(MenuEvent { id }) if id == "quit" => {
            match &radio_station.peek().torrent_client {
                ui::ResourceState::Loaded(client) => {
                    let _ = block_on(client.save(PathBuf::from("./data/torrents")));
                }
                _ => {}
            };
            ctx.exit();
        }
        _ => {}
    };

    let (manager, manager_tx) = AppManager::new(radio_station);

    launch(
        LaunchConfig::new()
            .with_tray(tray_icon, tray_handler)
            .with_future(async move |_| manager.run_manager().await)
            .with_window(WindowConfig::new_app(AkarekoApp::new(
                radio_station,
                router,
            ))),
    );

    Ok(())
}
