use std::path::PathBuf;

use freya::{
    prelude::*,
    query::{Mutation, Query, QueryStateData, use_mutation, use_query},
};

use crate::{
    db::index::{Index, tags::MangaTag},
    ui::{
        Route, RouteContext,
        components::ContentEntry,
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

        let title = label().text(self.index.title().clone());

        let index = self.index.clone();
        let add_chapter_press = move |_| {
            RouteContext::get().push(Route::AddMangaChapter {
                index: index.clone(),
            });
        };

        let top = rect()
            .horizontal()
            .child(ImageViewer::new(cover_holder).width(Size::px(400.)))
            .child(
                Button::new()
                    .child(svg(PLUS_ICON))
                    .on_press(add_chapter_press),
            );

        let chapters = match &*query.read().state() {
            QueryStateData::Settled {
                res: Ok(contents), ..
            } => {
                let chapters = contents
                    .iter()
                    .map(|c| ContentEntry::new(c.clone()).into_element());
                rect()
                    .vertical()
                    .child("Chapters")
                    .children(chapters)
                    .into_element()
            }
            QueryStateData::Pending | QueryStateData::Loading { .. } => {
                rect().child(CircularLoader::new()).into_element()
            }
            QueryStateData::Settled { res: Err(e), .. } => {
                rect().child(label().text(e.to_string())).into_element()
            }
        };

        rect().child(title).child(top).child(chapters)
    }
}
