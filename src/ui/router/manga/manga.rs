use freya::{
    prelude::*,
    query::{Mutation, Query, QueryStateData, use_mutation, use_query},
};

use crate::{
    db::index::{Index, tags::MangaTag},
    ui::{
        DEFAULT_CORNER_RADIUS, DEFAULT_PAGE_PADDING, Route, RouteContext, UNKNOWN_COVER,
        components::{ContentEntry, Spacer, svg_button},
        icons::{self},
        queries::{FetchContents, FollowContent, GetFollowContent},
    },
};

#[derive(PartialEq)]
pub struct Manga {
    pub index: Index<MangaTag>,
}
impl Component for Manga {
    fn render(&self) -> impl IntoElement {
        let contents_query = use_query(Query::new(
            self.index.hash().clone(),
            FetchContents::<MangaTag>::new(),
        ));
        let bookmark_query = use_query(Query::new(
            self.index.hash().clone(),
            GetFollowContent::<MangaTag>::new(),
        ));
        let bookmark_mut = use_mutation(Mutation::new(FollowContent::<MangaTag>::new()));

        let title = label().text(self.index.title().clone()).font_size(24);

        let index = self.index.clone();
        let add_chapter_press = move |_| {
            RouteContext::get().push(Route::AddMangaChapter {
                index: index.clone(),
            });
        };

        let follow_button = match &*bookmark_query.read().state() {
            QueryStateData::Pending => CircularLoader::new().into_element(),
            QueryStateData::Loading { .. } => CircularLoader::new().into_element(),
            QueryStateData::Settled { res, .. } => match res {
                Ok(Some(_)) => {
                    let index_hash = self.index.hash().clone();
                    Button::new()
                        .child(svg(icons::BOOK_BOOKMARK_ICON))
                        .on_press(move |_| {
                            bookmark_mut.mutate((index_hash.clone(), false));
                        })
                        .into_element()
                }
                Ok(None) => {
                    let index_hash = self.index.hash().clone();
                    Button::new()
                        .child(svg(icons::BOOK_BOOKMARK_ICON))
                        .on_press(move |_| {
                            bookmark_mut.mutate((index_hash.clone(), false));
                        })
                        .into_element()
                }
                Err(_) => "X".into_element(),
            },
        };

        let add_chapter_button =
            svg_button(icons::PLUS_ICON, 32., Color::BLACK).on_press(add_chapter_press);

        let top = rect()
            .horizontal()
            .child(
                ImageViewer::new(UNKNOWN_COVER)
                    .width(Size::px(400.))
                    .corner_radius(DEFAULT_CORNER_RADIUS),
            )
            .child(Spacer::horizontal(20.))
            .child(
                rect().child(title).child(
                    rect()
                        .horizontal()
                        .child(add_chapter_button)
                        .child(follow_button),
                ),
            );

        let chapters = match &*contents_query.read().state() {
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
