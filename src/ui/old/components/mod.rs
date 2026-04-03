use iced::{Color, Length, widget::svg};

use crate::ui::style;

pub mod modal;
pub mod sidebar;
pub mod toast;

pub fn svg_button<'a, Message: Clone>(
    svg_bytes: svg::Handle,
) -> iced::widget::button::Button<'static, Message> {
    iced::widget::button(
        iced::widget::svg(svg_bytes)
            .height(Length::Fixed(24.0))
            .width(Length::Fixed(24.0))
            .style(|_, _| iced::widget::svg::Style {
                color: Some(Color::WHITE),
            }),
    )
    .style(style::icon_button)
}
