use std::path::PathBuf;

use freya::{
    prelude::*,
    query::{Mutation, Query, QueryStateData, use_mutation, use_query},
};

use crate::{
    db::index::{Index, tags::MangaTag},
    ui::{
        DEFAULT_CORNER_RADIUS, DEFAULT_PAGE_PADDING, Route, RouteContext,
        components::{AkLayers, ContentEntry, Spacer, svg_button},
        icons::PLUS_ICON,
        queries::{FetchContents, UpdateContentProgress},
    },
};

#[derive(PartialEq)]
pub struct Manga {
    pub index: Index<MangaTag>,
}
impl Component for Manga {
    fn render(&self) -> impl IntoElement {
        let query = use_query(Query::new(
            self.index.hash().clone(),
            FetchContents::<MangaTag>::new(),
        ));

        let cover_holder: ImageSource = PathBuf::from("./assets/placeholder_cover.png").into();

        let title = label().text(self.index.title().clone()).font_size(24);

        let index = self.index.clone();
        let add_chapter_press = move |_| {
            RouteContext::get().push(Route::AddMangaChapter {
                index: index.clone(),
            });
        };

        let top = rect()
            .horizontal()
            .child(
                ImageViewer::new(cover_holder)
                    .width(Size::px(400.))
                    .corner_radius(DEFAULT_CORNER_RADIUS),
            )
            .child(Spacer::horizontal(20.))
            .child(
                rect()
                    .child(title)
                    .child(svg_button(PLUS_ICON, 32., Color::BLACK).on_press(add_chapter_press)),
            );

        let chapters = match &*query.read().state() {
            QueryStateData::Settled {
                res: Ok(contents), ..
            } => {
                let chapters = contents
                    .iter()
                    .map(|c| ContentEntry::new(c.clone()).into_element());
                rect().vertical().children(chapters).into_element()
            }
            QueryStateData::Pending | QueryStateData::Loading { .. } => {
                rect().child(CircularLoader::new()).into_element()
            }
            QueryStateData::Settled { res: Err(e), .. } => {
                rect().child(label().text(e.to_string())).into_element()
            }
        };

        rect()
            .child(top)
            .child(Spacer::vertical(50.))
            .child(chapters)
            .padding(DEFAULT_PAGE_PADDING)
    }
}
