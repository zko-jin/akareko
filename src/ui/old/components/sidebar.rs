use crate::ui::{components::svg_button, icons::ARROW_LEFT_ICON, message::Message};

pub fn sidebar<'a>(can_pop: bool) -> iced::Element<'a, Message> {
    let back_message = if can_pop {
        Some(Message::BackHistory)
    } else {
        None
    };

    iced::widget::container(
        iced::widget::column![iced::widget::row![
            svg_button(ARROW_LEFT_ICON.clone()).on_press_maybe(back_message),
        ]]
        .spacing(10),
    )
    .width(iced::Length::Fill)
    .height(iced::Length::Fill)
    // .style(style::sidebar)
    .into()
}
