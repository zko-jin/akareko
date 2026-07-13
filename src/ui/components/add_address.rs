use freya::{prelude::*, query::*};

use crate::{
    db::user::I2PAddress,
    ui::{
        components::svg_button,
        icons::{self},
        queries::{self},
    },
};

#[derive(PartialEq)]
pub struct AddAddress;

impl Component for AddAddress {
    fn render(&self) -> impl IntoElement {
        let address_str = use_state(String::new);
        let add_address = use_mutation(Mutation::new(queries::AddAddress));

        let add_address_button =
            svg_button(icons::PLUS_ICON, 20., Color::WHITE).on_press(move |_| {
                add_address.mutate(I2PAddress::new(address_str.read().clone()));
            });

        let address_input = Input::new(address_str).placeholder("Enter address");

        rect().child(address_input).child(add_address_button)
    }
}
