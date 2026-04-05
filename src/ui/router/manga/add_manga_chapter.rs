use freya::{prelude::*, query::*, radio::use_radio};

use crate::{
    db::{
        Magnet,
        index::{
            Index,
            content::Content,
            tags::{MangaChapter, MangaTag},
        },
    },
    helpers::Language,
    types::Timestamp,
    ui::{AppChannel, ResourceState, queries::AddIndexContent},
};

#[derive(PartialEq)]
pub struct AddMangaChapter {
    pub index: Index<MangaTag>,
}
impl Component for AddMangaChapter {
    fn render(&self) -> impl IntoElement {
        let title = use_state(String::new);
        let path = use_state(String::new);
        let magnet_link = use_state(String::new);
        let enumeration = use_state(|| "1".to_string());
        let state = use_radio(AppChannel::Config);

        let mutation = use_mutation(Mutation::new(AddIndexContent::<MangaTag>::new()));

        let hash = self.index.hash().clone();

        rect()
            .child(Input::new(title).placeholder("Title"))
            .child(Input::new(magnet_link).placeholder("Magnet Link"))
            .child(Input::new(path).placeholder("Path"))
            .child(
                Input::new(enumeration)
                    .placeholder("Enumeration")
                    .on_validate(|v: InputValidator| {
                        let r = v.text().parse::<f32>();
                        v.set_valid(r.is_ok());
                    })
                    .text_align(TextAlign::Left),
            )
            .child(Button::new().child("Add").on_press(move |_| {
                if let ResourceState::Loaded(c) = &state.read().config {
                    mutation.mutate(Content::new_signed(
                        hash.clone(),
                        Timestamp::now(),
                        Magnet(magnet_link.read().clone()),
                        path.read().clone(),
                        title.read().clone(),
                        0.0,
                        None,
                        MangaChapter::new(Language::Unknown),
                        c.private_key(),
                    ));
                }
                // RouterContext::get().push(Route::Manga { hash: hash.clone()
                // });
            }))
    }
}
