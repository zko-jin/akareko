use const_format::formatcp;
use freya::{prelude::*, radio::use_radio};

use crate::{
    config::{AkarekoConfig, DEFAULT_SAM_PORT},
    ui::{AppChannel, DEFAULT_PAGE_PADDING, ResourceState},
};

#[derive(PartialEq)]
pub struct Settings;

const DEFAULT_SAM_PORT_STR: &'static str = formatcp!("{}", DEFAULT_SAM_PORT);
impl Component for Settings {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio(AppChannel::Config);
        let mut new_config = use_state(|| radio.read().config.unwrap_ref().clone());

        let sam_port_string = use_state(move || {
            let sam_port = new_config.read().sam_port();
            sam_port.to_string()
        });

        let dev_mode_switch = Switch::new()
            .toggled(new_config.read().dev_mode())
            .on_toggle(move |_| {
                let mut config = new_config.write();
                let dev_mode = !config.dev_mode();
                config.set_dev_mode(dev_mode);
            });

        let sam_port_input = rect()
            .spacing(10.)
            .horizontal()
            .cross_align(Alignment::Center)
            .child("SAM Port:")
            .child(
                Input::new(sam_port_string)
                    .placeholder(DEFAULT_SAM_PORT_STR)
                    .on_validate(move |v: InputValidator| {
                        if v.text().is_empty() {
                            new_config.write().set_sam_port(DEFAULT_SAM_PORT);
                            return;
                        }

                        let r = v.text().parse::<u16>();
                        if let Ok(port) = r {
                            new_config.write().set_sam_port(port);
                            return;
                        }

                        v.set_valid(false);
                    }),
            );

        let i2p_configs = rect()
            .child(label().text("I2P").font_size(32))
            .child(
                rect()
                    .spacing(20.)
                    .horizontal()
                    .child("I2P Address:")
                    .child(new_config.read().eepsite_address().inner().clone()),
            )
            .child(sam_port_input);

        let is_dirty = *radio.read().config.unwrap_ref() != *new_config.read();

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
                                radio.write().config =
                                    ResourceState::Loaded(new_config.read().cloned());
                            }),
                    )
                    .child(Button::new().child("Cancel")),
            )
    }
}
