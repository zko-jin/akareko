use freya::{prelude::*, query::*, radio::use_radio, router::RouterContext};

use crate::{
    db::index::tags::MangaTag,
    ui::{
        DEFAULT_PAGE_PADDING, IndexComponent,
        icons::PLUS_ICON,
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

        rect()
            .padding(DEFAULT_PAGE_PADDING)
            .child(
                Button::new()
                    .child(svg(PLUS_ICON))
                    .on_press(|_| RouteContext::get().push(Route::AddManga)),
            )
            .child(manga_list)
    }
}
