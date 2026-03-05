#![allow(dead_code)]

use clap::Parser;
use tracing::info;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::ui::AppState;

mod config;
mod db;
mod errors;
mod hash;
mod helpers;
mod server;
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

    iced::daemon(AppState::boot, AppState::update, AppState::view)
        .subscription(AppState::subscription)
        .theme(|s: &AppState, _| s.theme())
        .run()
        .unwrap();

    Ok(())
}
