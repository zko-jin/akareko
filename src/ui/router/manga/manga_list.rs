use freya::{prelude::*, query::*};

use crate::{
    db::index::tags::MangaTag,
    ui::{
        DEFAULT_CORNER_RADIUS, DEFAULT_PAGE_PADDING, IndexComponent,
        components::svg_button,
        icons::{self, PLUS_ICON},
        queries::FetchIndexes,
        router::{Route, RouteContext},
    },
};

#[derive(PartialEq)]
pub struct MangaList;
impl Component for MangaList {
    fn render(&self) -> impl IntoElement {
        let manga_query = use_query(Query::new((), FetchIndexes::<MangaTag>::new()));

        let manga_list = match &*manga_query.read().state() {
            QueryStateData::Pending => rect().child(CircularLoader::new()),
            QueryStateData::Loading { .. } => rect().child(CircularLoader::new()),
            QueryStateData::Settled { res, .. } => match res {
                Ok(res) => {
                    let children: Vec<Element> = res
                        .into_iter()
                        .map(|i| IndexComponent { index: i.clone() }.into_element())
                        .collect();

                    rect().children(children)
                }
                Err(e) => rect().child(label().text(e.to_string())),
            },
        };

        let search_string = use_state(String::new);

        let search_bar = Input::new(search_string)
            .placeholder("Search")
            .leading(svg_button(icons::CHECK_CIRCLE_ICON, 24., Color::BLACK))
            .corner_radius(DEFAULT_CORNER_RADIUS)
            .width(Size::Fill);

        rect()
            .spacing(10.)
            .padding(DEFAULT_PAGE_PADDING)
            .width(Size::Fill)
            .child(search_bar)
            .child(
                Button::new()
                    .child(svg(PLUS_ICON))
                    .on_press(|_| RouteContext::get().push(Route::AddManga)),
            )
            .child(manga_list)
    }
}
