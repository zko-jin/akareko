use freya::prelude::*;

use crate::ui::{Route, RouteContext};

pub fn layout_button(route: Route) -> impl IntoElement {
    let selected = *RouteContext::get().state().route() == route;

    Button::new()
        .child(
            label()
                .text(route.name())
                .text_align(TextAlign::End)
                .width(Size::Fill),
        )
        .on_press(move |_| {
            RouteContext::get().push(route.clone());
        })
        .width(Size::Fill)
        .hover_background(Color::from_af32rgb(0.5, 0, 0, 0))
        .maybe(selected, |b| b.background(Color::WHITE))
        .corner_radius(0.)
        .flat()
        .expanded()
}
