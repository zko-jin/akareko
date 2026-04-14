use crate::db::index::content::Content;
use crate::db::index::tags::MangaTag;
use crate::db::index::{Index, content::ExternalContent};
use crate::helpers::LiFo;
use freya::prelude::*;

mod home;
mod settings;
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
mod torrents;
use torrents::Torrents;

use home::Home;
use manga::{AddManga, AddMangaChapter, ChapterViewer, Manga, MangaList};
use settings::Settings;

#[derive(Clone, PartialEq)]
pub enum Route {
    // #[layout(Layout)]
    // #[route("/")]
    Home,

    // #[nest("/manga")]
    // #[route("/")]
    MangaList,
    // #[route("/:hash")]
    Manga {
        index: Index<MangaTag>,
    },
    // #[route("/add")]
    AddManga,
    // #[route("/:hash/add")]
    AddMangaChapter {
        index: Index<MangaTag>,
    },
    // #[route("/chapter/:signature")]
    ChapterViewerInternal {
        content: Content<MangaTag>,
    },
    ChapterViewerExternal {
        content: Content<MangaTag, ExternalContent>,
    },
    Settings,
    Torrents,
}

impl Route {
    pub fn name(&self) -> &'static str {
        match self {
            Route::Home => "Home",
            Route::MangaList => "Mangas",
            Route::Manga { .. } => "",
            Route::AddManga => "Add Manga",
            Route::AddMangaChapter { .. } => "",
            Route::ChapterViewerInternal { .. } => "Chapter Viewer",
            Route::ChapterViewerExternal { .. } => "Chapter Viewer",
            Route::Settings => "Settings",
            Route::Torrents => "Torrents",
        }
    }
}

pub struct RouteState {
    route: Route,
    history: LiFo<Route, 10>,
}

impl RouteState {
    pub fn route(&self) -> &Route {
        &self.route
    }

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

    pub fn state(&self) -> ReadRef<'_, RouteState> {
        self.state.read()
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
            Route::ChapterViewerInternal { content } => ChapterViewer {
                content: content.clone(),
            }
            .into_element(),
            Route::ChapterViewerExternal { content } => ChapterViewer {
                content: content.clone(),
            }
            .into_element(),
            Route::Settings => Settings.into_element(),
            Route::Torrents => Torrents.into_element(),
        }
    }
}
