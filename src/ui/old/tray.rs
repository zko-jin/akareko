use trayicon::{Icon, MenuBuilder, TrayIcon, TrayIconBuilder};

use crate::ui::TrayIconMessage;

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
