use const_format::formatcp;
use freya::{prelude::*, radio::use_radio};

use crate::{
    config::DEFAULT_SAM_PORT,
    ui::{AppChannel, DEFAULT_PAGE_PADDING},
};

#[derive(PartialEq)]
pub struct Config;

const DEFAULT_SAM_PORT_STR: &'static str = formatcp!("{}", DEFAULT_SAM_PORT);
impl Component for Config {
    fn render(&self) -> impl IntoElement {
        let mut radio = use_radio(AppChannel::Config);
        let sam_port_string = use_state(move || {
            let sam_port = radio.read().config.unwrap_ref().sam_port();
            if sam_port == DEFAULT_SAM_PORT {
                String::new()
            } else {
                sam_port.to_string()
            }
        });

        let dev_mode_switch = Switch::new()
            .toggled(radio.read().config.unwrap_ref().dev_mode())
            .on_toggle(move |_| {
                let mut w = radio.write();
                let config = w.config.mut_unwrap_ref();
                config.set_dev_mode(!config.dev_mode());
            });

        let sam_port_input = rect()
            .spacing(10.)
            .horizontal()
            .cross_align(Alignment::Center)
            .child("SAM Port:")
            .child(
                Input::new(sam_port_string)
                    .placeholder(DEFAULT_SAM_PORT_STR)
                    .on_validate(|v: InputValidator| {
                        if v.text().is_empty() {
                            v.set_valid(true);
                            return;
                        }
                        let r = v.text().parse::<u16>();
                        v.set_valid(r.is_ok());
                    }),
            );

        let i2p_configs = rect()
            .child(label().text("I2P").font_size(32))
            .child(
                rect()
                    .spacing(20.)
                    .horizontal()
                    .child("I2P Address:")
                    .child(
                        radio
                            .read()
                            .config
                            .unwrap_ref()
                            .eepsite_address()
                            .inner()
                            .clone(),
                    ),
            )
            .child(sam_port_input);

        rect()
            .padding(DEFAULT_PAGE_PADDING)
            .spacing(15.)
            .child(label().text("Settings").font_size(48))
            .child(i2p_configs)
            .child(dev_mode_switch)
            .child(
                rect()
                    .horizontal()
                    .child(Button::new().child("Save"))
                    .child(Button::new().child("Cancel")),
            )
    }
}
