use futures::SinkExt;
use iced::Subscription;
use tokio::sync::mpsc;
use tracing::error;
use trayicon::{Icon, MenuBuilder, TrayIcon, TrayIconBuilder};

use crate::ui::{TrayIconMessage, message::Message};

pub fn initialize_tray_icon() -> TrayIcon<TrayIconMessage> {
    const ICON: &'static [u8] = include_bytes!("../../assets/tray_icon.ico");

    let tray_icon = TrayIconBuilder::new()
        .title("Akareko")
        .tooltip("Akareko Desktop Client")
        .icon(Icon::from_buffer(ICON, None, None).unwrap())
        .menu(
            MenuBuilder::new()
                .item("Open", TrayIconMessage::OpenWindow)
                .item("Quit", TrayIconMessage::Exit),
        )
        .build()
        .unwrap();

    tray_icon
}

// pub fn tray_worker() -> impl iced::futures::Stream<Item = Message> {
//     iced::stream::channel(
//         8,
//         |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
//             let (tx, mut rx) = tokio::sync::mpsc::channel(8);

//             match output.send(Message::TrayIconReady(tx)).await {
//                 Ok(()) => {}
//                 Err(e) => {
//                     error!("Error initializing tray icon: {}", e);
//                 }
//             };

//             loop {
//                 let tray_message = match rx.recv().await {
//                     Some(m) => m,
//                     None => break,
//                 };

//                 match output.send(Message::TrayIconMessage(tray_message)).await {
//                     Ok(()) => {}
//                     Err(e) => {
//                         if e.is_disconnected() {
//                             error!("Disconnected from tray output!");
//                         } else if e.is_full() {
//                             error!("Tray output is full!");
//                         }
//                     }
//                 };
//             }
//         },
//     )
// }
