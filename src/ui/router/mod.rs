use std::path::PathBuf;

use crate::db::index::Index;
use crate::db::index::content::Content;
use crate::db::index::tags::MangaTag;
use crate::helpers::LiFo;
use crate::types::{Hash, Signature};
use crate::ui::Layout;
use freya::prelude::*;
use freya::router::Routable;

mod home;
mod manga {
    mod manga;
    pub use manga::Manga;
    mod manga_list;
    pub use manga_list::MangaList;
    mod add_manga;
    pub use add_manga::AddManga;
    mod add_manga_chapter;
    pub use add_manga_chapter::AddMangaChapter;
    mod chapter_viewer;
    pub use chapter_viewer::ChapterViewer;
}

use home::Home;
use manga::{AddManga, AddMangaChapter, ChapterViewer, Manga, MangaList};

#[derive(Clone, PartialEq)]
pub enum Route {
    // #[layout(Layout)]
    // #[route("/")]
    Home,

    // #[nest("/manga")]
    // #[route("/")]
    MangaList,
    // #[route("/:hash")]
    Manga { index: Index<MangaTag> },
    // #[route("/add")]
    AddManga,
    // #[route("/:hash/add")]
    AddMangaChapter { index: Index<MangaTag> },
    // #[route("/chapter/:signature")]
    ChapterViewer { content: Content<MangaTag> },
}

pub struct RouteState {
    route: Route,
    history: LiFo<Route, 10>,
}

impl RouteState {
    fn change_route(&mut self, route: Route) {
        let old = std::mem::replace(&mut self.route, route);
        self.history.push(old);
    }
}

#[derive(Clone, Copy)]
pub struct RouteContext {
    state: State<RouteState>,
}

impl RouteContext {
    pub fn create_global() -> Self {
        Self {
            state: State::create_global(RouteState {
                route: Route::Home,
                history: LiFo::new(),
            }),
        }
    }

    pub fn get() -> Self {
        consume_context()
    }

    pub fn push(&mut self, route: Route) {
        self.state.write().change_route(route);
    }

    pub fn go_back(&mut self) {
        let mut state = self.state.write();
        if let Some(route) = state.history.pop() {
            state.change_route(route);
        }
    }

    pub fn can_go_back(&self) -> bool {
        self.state.read().history.can_pop()
    }
}

#[derive(PartialEq)]
pub struct RouteComponent;

impl Component for RouteComponent {
    fn render(&self) -> impl IntoElement {
        let route_context = RouteContext::get();
        route_context.state.read().route.clone()
    }
}

impl Component for Route {
    fn render(&self) -> impl IntoElement {
        match self {
            Route::Home => Home.into_element(),
            Route::MangaList => MangaList.into_element(),
            Route::Manga { index } => Manga {
                index: index.clone(),
            }
            .into_element(),
            Route::AddManga => AddManga.into_element(),
            Route::AddMangaChapter { index } => AddMangaChapter {
                index: index.clone(),
            }
            .into_element(),
            Route::ChapterViewer { content } => ChapterViewer {
                content: content.clone(),
            }
            .into_element(),
        }
    }
}
