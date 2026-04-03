use crate::ui::router::{Route, RouteContext};
use freya::prelude::*;

#[derive(PartialEq)]
pub struct Home;
impl Component for Home {
    fn render(&self) -> impl IntoElement {
        rect().child(Button::new().flat().child("Mangas").on_press(|_| {
            RouteContext::get().push(Route::MangaList);
        }))
    }
}
