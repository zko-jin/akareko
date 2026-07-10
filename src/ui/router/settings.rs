use const_format::formatcp;
use freya::prelude::*;

use crate::{
    config::DEFAULT_SAM_TCP_PORT,
    ui::{AppResources, DEFAULT_PAGE_PADDING},
};

#[derive(PartialEq)]
pub struct SettingsPage;

const DEFAULT_SAM_TCP_PORT_STR: &'static str = formatcp!("{}", DEFAULT_SAM_TCP_PORT);
impl Component for SettingsPage {
    fn render(&self) -> impl IntoElement {
        let mut new_config = use_state(|| AppResources::get_config().unwrap_ref().clone());

        // let sam_port_string = use_state(move || {
        //     let sam_port = new_config.read().sam_tcp_port();
        //     sam_port.to_string()
        // });

        let dev_mode_switch = Switch::new()
            .toggled(new_config.read().dev_mode())
            .on_toggle(move |_| {
                let mut config = new_config.write();
                let dev_mode = !config.dev_mode();
                config.set_dev_mode(dev_mode);
            });

        // let sam_port_input = rect()
        //     .spacing(10.)
        //     .horizontal()
        //     .cross_align(Alignment::Center)
        //     .child("SAM Port:")
        //     .child(
        //         Input::new(sam_port_string)
        //             .placeholder(DEFAULT_SAM_TCP_PORT_STR)
        //             .on_validate(move |v: InputValidator| {
        //                 if v.text().is_empty() {
        //
        // new_config.write().set_sam_tcp_port(DEFAULT_SAM_TCP_PORT);
        //                     return;
        //                 }

        //                 let r = v.text().parse::<u16>();
        //                 if let Ok(port) = r {
        //                     new_config.write().set_sam_tcp_port(port);
        //                     return;
        //                 }

        //                 v.set_valid(false);
        //             }),
        //     );

        let i2p_configs = rect().child(label().text("I2P").font_size(32)).child(
            rect()
                .spacing(20.)
                .horizontal()
                .child("I2P Address:")
                .child(new_config.read().eepsite_address().inner().clone()),
        );
        // .child(sam_port_input);

        let is_dirty = *AppResources::get_config().unwrap_ref() != *new_config.read();

        rect()
            .padding(DEFAULT_PAGE_PADDING)
            .spacing(15.)
            .child(label().text("Settings").font_size(48))
            .child(i2p_configs)
            .child(dev_mode_switch)
            .child(
                rect()
                    .horizontal()
                    .child(
                        Button::new()
                            .child("Save")
                            .enabled(is_dirty)
                            .on_press(move |_| {
                                // *radio.write().config_mut() =
                                //     ResourceState::Loaded(new_config.read().
                                // cloned());
                            }),
                    )
                    .child(Button::new().child("Cancel")),
            )
    }
}
