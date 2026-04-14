#![allow(dead_code)]

use std::path::PathBuf;

use clap::Parser;
use freya::{
    prelude::*,
    radio::RadioStation,
    tray::{
        Icon, TrayEvent, TrayIconBuilder, TrayIconEvent,
        menu::{Menu, MenuEvent, MenuItem},
    },
};
use futures::executor::block_on;
use tracing::info;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::ui::{
    AkarekoApp, AppChannel, AppState, AppWindowType, RouteContext,
    app_manager::{AppManager, Event},
};

// use crate::ui::AppState;

mod clients;
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
    ///   Start the application in minimized state.
    #[arg(long)]
    minimized: bool,
}

fn main() -> Result<(), ()> {
    let args = CliArgs::parse();

    // ==================== Tracing ====================
    let borrowed_format_items =
        time::format_description::parse(":[minute]:[second]").expect("Cataplum");
    let format = borrowed_format_items;

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

    let mut app_state = AppState::new();
    if !args.minimized {
        app_state.windows_state.try_add_window(AppWindowType::Main);
    }
    let mut radio_station = RadioStation::<AppState, AppChannel>::create_global(app_state);

    let router = RouteContext::create_global();

    let (manager, manager_tx) = AppManager::new(radio_station);
    let app = AkarekoApp::new(radio_station, router);

    let manager_tx_tray = manager_tx.clone();
    let tray_handler = move |ev, mut ctx: RendererContext| match ev {
        TrayEvent::Icon(TrayIconEvent::Click { .. }) => {
            // TODO: Deduplicate code
            let can_open_window = radio_station
                .write_channel(AppChannel::Window)
                .windows_state
                .try_add_window(AppWindowType::Main);

            if can_open_window {
                let manager_tx = manager_tx_tray.clone();
                ctx.launch_window(WindowConfig::new_app(app).with_on_close(move |_, _| {
                    manager_tx.send(Event::RemoveMainWindow).unwrap();
                    CloseDecision::Close
                }));
            }
        }
        TrayEvent::Menu(MenuEvent { id }) if id == "open" => {
            let can_open_window = radio_station
                .write_channel(AppChannel::Window)
                .windows_state
                .try_add_window(AppWindowType::Main);

            if can_open_window {
                let manager_tx = manager_tx_tray.clone();
                ctx.launch_window(WindowConfig::new_app(app).with_on_close(move |_, _| {
                    manager_tx.send(Event::RemoveMainWindow).unwrap();
                    CloseDecision::Close
                }));
            }
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
    let mut launch_config = LaunchConfig::new()
        .with_tray(tray_icon, tray_handler)
        .with_future(async move |_| manager.run_manager().await)
        .with_exit_on_close(false);

    if !args.minimized {
        launch_config =
            launch_config.with_window(WindowConfig::new_app(app).with_on_close(move |_, _| {
                manager_tx.send(Event::RemoveMainWindow).unwrap();
                CloseDecision::Close
            }));
    }

    launch(launch_config);

    Ok(())
}
