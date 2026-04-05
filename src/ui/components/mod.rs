use freya::prelude::*;

mod circular_progress_bar;
mod content_entry;
mod layout_button;

pub use content_entry::ContentEntry;
pub use layout_button::layout_button;

pub enum AkLayers {
    Frame,
    Sidebars,
}

impl Into<Layer> for AkLayers {
    fn into(self) -> Layer {
        match self {
            AkLayers::Sidebars => Layer::RelativeOverlay(5),
            AkLayers::Frame => Layer::RelativeOverlay(100),
        }
    }
}

pub fn svg_button(icon: &'static [u8], size: f32, color: Color) -> Button {
    Button::new()
        .child(
            svg(icon)
                .width(Size::px(size))
                .height(Size::px(size))
                .color(color),
        )
        .padding(0.)
        .flat()
        .compact()
}

#[derive(PartialEq)]
pub struct Spacer {
    width: Size,
    height: Size,
}
impl Component for Spacer {
    fn render(&self) -> impl IntoElement {
        rect().width(self.width.clone()).height(self.height.clone())
    }
}

impl Spacer {
    pub fn new(width: Size, height: Size) -> Spacer {
        Spacer { width, height }
    }

    pub fn vertical(height: f32) -> Spacer {
        Spacer {
            width: Size::default(),
            height: Size::px(height),
        }
    }

    pub fn horizontal(width: f32) -> Spacer {
        Spacer {
            height: Size::default(),
            width: Size::px(width),
        }
    }

    pub fn horizontal_fill() -> Spacer {
        Spacer {
            height: Size::default(),
            width: Size::flex(1.),
        }
    }
}
