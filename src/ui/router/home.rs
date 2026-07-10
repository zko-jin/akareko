use crate::{
    daemon::resource_state::ResourceState,
    ui::{AppResources, DEFAULT_CORNER_RADIUS, DEFAULT_PAGE_PADDING, icons},
};
use freya::prelude::*;

#[derive(PartialEq)]
pub struct Home;
impl Component for Home {
    fn render(&self) -> impl IntoElement {
        fn render_status<T, E>(name: &'static str, state: &ResourceState<T, E>) -> Element {
            let icon = match state {
                ResourceState::Pending => svg(icons::CIRCLE)
                    .fill(Color::LIGHT_GRAY)
                    .stroke_width(12.)
                    .stroke(Color::BLACK)
                    .height(Size::px(12.))
                    .into_element(),
                ResourceState::Error(_) => svg(icons::CIRCLE)
                    .fill(Color::RED)
                    .stroke_width(12.)
                    .stroke(Color::BLACK)
                    .height(Size::px(12.))
                    .into_element(),
                ResourceState::Loaded(_) => svg(icons::CIRCLE)
                    .fill(Color::GREEN)
                    .stroke_width(12.)
                    .stroke(Color::BLACK)
                    .height(Size::px(12.))
                    .into_element(),
                ResourceState::Loading => svg(icons::CIRCLE)
                    .fill(Color::YELLOW)
                    .stroke_width(12.)
                    .stroke(Color::BLACK)
                    .height(Size::px(12.))
                    .into_element(),
            };

            rect()
                .horizontal()
                .content(Content::Flex)
                .cross_align(Alignment::Center)
                .padding(10.)
                .child(label().text(name).width(Size::flex(1.)))
                .child(icon)
                .into_element()
        }

        let status = rect()
            .border(Some(Border::new().width(2.).fill(Color::DARK_GRAY)))
            .width(Size::px(150.))
            .corner_radius(DEFAULT_CORNER_RADIUS)
            .children([
                render_status("Repositories", &AppResources::get_repositories()),
                rect()
                    .width(Size::Fill)
                    .height(Size::px(2.))
                    .background(Color::GRAY)
                    .into_element(),
                render_status("Torrent Client", &AppResources::get_torrent_client()),
                // rect()
                //     .width(Size::Fill)
                //     .height(Size::px(2.))
                //     .background(Color::GRAY)
                //     .into_element(),
                // render_status("Server", &AppResources::get_server()),
                rect()
                    .width(Size::Fill)
                    .height(Size::px(2.))
                    .background(Color::GRAY)
                    .into_element(),
                render_status("Client", &AppResources::get_client()),
            ]);

        rect().padding(DEFAULT_PAGE_PADDING).child(
            rect()
                .center()
                .child(label().text("Status").font_size(32.))
                .child(status),
        )
    }
}
