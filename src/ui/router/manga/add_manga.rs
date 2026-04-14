use std::time::Duration;

use freya::{
    prelude::*,
    query::{Mutation, use_mutation},
    radio::use_radio,
};

use crate::{
    db::index::{Index, IndexLinks, tags::MangaTag},
    ui::{AppChannel, ResourceState, queries::AddIndex},
};

#[derive(PartialEq)]
pub struct AddManga;
impl Component for AddManga {
    fn render(&self) -> impl IntoElement {
        let title = use_state(String::new);
        let mangadex_id = use_state(String::new);
        let state = use_radio(AppChannel::Config);

        let mut selected = use_state(|| None::<CalendarDate>);
        let mut view_date = use_state(|| CalendarDate::new(2025, 1, 1));
        let calendar = Calendar::new()
            .selected(selected())
            .view_date(view_date())
            .on_change(move |date| selected.set(Some(date)))
            .on_view_change(move |date| view_date.set(date));

        let links = rect()
            .vertical()
            .child(Input::new(mangadex_id).placeholder("a1c7c817-4e59-43b7-9365-09675a149a6f"));

        let mutation = use_mutation(
            Mutation::new(AddIndex::<MangaTag>::new()).clean_time(Duration::from_secs(5)),
        );

        rect()
            .child(Input::new(title).placeholder("Title"))
            .child(Button::new().child("Add").on_press(move |_| {
                if let ResourceState::Loaded(c) = &state.read().config {
                    let mangadex = mangadex_id.read().cloned();
                    let mangadex = if mangadex.is_empty() {
                        None
                    } else {
                        Some(mangadex.try_into().unwrap())
                    };

                    mutation.mutate(Index::new_signed(
                        title.read().clone(),
                        0,
                        IndexLinks {
                            myanimelist: None,
                            mangadex,
                        },
                        c.private_key(),
                    ));
                }

                // RouterContext::get().push(Route::MangaList);
            }))
            .child(calendar)
            .child(links)
    }
}
