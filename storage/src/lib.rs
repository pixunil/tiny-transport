#![allow(clippy::zero_prefixed_literal)]

mod dataset;
mod line;
mod node;
mod station;
mod train;

pub use crate::dataset::Dataset;
pub use crate::line::Line;
pub use crate::node::{Kind as NodeKind, Node};
pub use crate::station::Station;
pub use crate::train::Train;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    pub use crate::dataset::fixtures as datasets;
    pub use crate::line::fixtures as lines;
    pub use crate::node::fixtures as nodes;
    pub use crate::station::fixtures as stations;
    pub use crate::train::fixtures as trains;
}
