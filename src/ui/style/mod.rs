use iced::{Color, Theme, widget::button};

pub fn icon_button(_: &Theme, _: button::Status) -> button::Style {
    button::Style {
        // background: Some(Color::TRANSPARENT.into()),
        background: None,
        text_color: Color::WHITE,
        ..Default::default()
    }
}
