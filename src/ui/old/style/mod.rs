use iced::{Color, Theme as IcedTheme, widget::button};

mod theme;
pub use theme::Theme;

pub fn icon_button(_: &IcedTheme, _: button::Status) -> button::Style {
    button::Style {
        // background: Some(Color::TRANSPARENT.into()),
        background: None,
        text_color: Color::WHITE,
        ..Default::default()
    }
}
