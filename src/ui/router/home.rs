use crate::ui::{AppChannel, DEFAULT_CORNER_RADIUS, DEFAULT_PAGE_PADDING, ResourceState};
use freya::{prelude::*, radio::use_radio};

#[derive(PartialEq)]
pub struct Home;
impl Component for Home {
    fn render(&self) -> impl IntoElement {
        let radio = use_radio(AppChannel::Status);

        fn render_status<T, E>(name: &'static str, state: &ResourceState<T, E>) -> Element {
            let icon = match state {
                ResourceState::Pending => "...",
                ResourceState::Error(_) => "X",
                ResourceState::Loaded(_) => "✓",
                ResourceState::Loading => "⏳",
            };

            rect()
                .horizontal()
                .content(Content::Flex)
                .cross_align(Alignment::Center)
                .padding(10.)
                .child(label().text(name).width(Size::flex(1.)))
                .child(label().text(icon))
                .into_element()
        }

        let status = rect()
            .border(Some(Border::new().width(2.).fill(Color::DARK_GRAY)))
            .width(Size::px(150.))
            .corner_radius(DEFAULT_CORNER_RADIUS)
            .children([
                render_status("Repositories", &radio.read().repositories),
                rect()
                    .width(Size::Fill)
                    .height(Size::px(2.))
                    .background(Color::GRAY)
                    .into_element(),
                render_status("Torrent Client", &radio.read().torrent_client),
                rect()
                    .width(Size::Fill)
                    .height(Size::px(2.))
                    .background(Color::GRAY)
                    .into_element(),
                render_status("Server", &radio.read().server),
                rect()
                    .width(Size::Fill)
                    .height(Size::px(2.))
                    .background(Color::GRAY)
                    .into_element(),
                render_status("Client", &radio.read().client),
            ]);

        rect().padding(DEFAULT_PAGE_PADDING).child(
            rect()
                .center()
                .child(label().text("Status").font_size(32.))
                .child(status),
        )
    }
}
