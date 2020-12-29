#![allow(clippy::module_inception, clippy::zero_prefixed_literal)]

mod color;
mod dataset;
mod direction;
pub mod line;
pub mod path;
pub mod station;
mod train;

pub use crate::color::Color;
pub use crate::dataset::Dataset;
pub use crate::direction::Direction;
pub use crate::line::Line;
pub use crate::station::Station;
pub use crate::train::Train;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    pub use crate::dataset::fixtures as datasets;
    pub use crate::line::fixtures::*;
    pub use crate::path::fixtures::{paths, segments};
    pub use crate::station::fixtures as stations;
    pub use crate::train::fixtures as trains;
}
