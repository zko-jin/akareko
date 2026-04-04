mod circular_progress_bar;
mod content_entry;
pub use content_entry::ContentEntry;
use freya::prelude::Layer;

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
