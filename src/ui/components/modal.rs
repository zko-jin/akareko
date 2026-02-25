use iced::{
    Color, Element, Task,
    widget::{center, container, mouse_area, opaque, stack, text},
};

use crate::ui::{
    AppState, Message,
    components::modal::add_who::{AddWhoModal, AddWhoModalMessage},
};

pub mod add_who;

#[derive(Debug, Clone)]
pub enum Modal {
    AddWho(AddWhoModal),
}

#[derive(Debug, Clone)]
pub enum ModalMessage {
    AddWho(AddWhoModalMessage),
}

impl Modal {
    pub fn view(state: &AppState) -> Element<'_, Message> {
        if state.modal.is_none() {
            return text("Empty modal being viewed").into();
        }

        match state.modal.as_ref().unwrap() {
            Modal::AddWho(m) => m.view(state),
        }
    }

    pub fn update(message: ModalMessage, state: &mut AppState) -> Task<Message> {
        match message {
            ModalMessage::AddWho(m) => AddWhoModal::update(m, state),
        }
    }
}

pub fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    on_blur: Message,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.8,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(on_blur)
        )
    ]
    .into()
}
