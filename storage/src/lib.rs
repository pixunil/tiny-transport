#![allow(clippy::zero_prefixed_literal)]

mod dataset;
mod line;
mod path;
mod schedule;
mod station;
mod train;

pub use crate::dataset::Dataset;
pub use crate::line::Line;
pub use crate::path::{Node, NodeKind, Segment, SegmentRef, SegmentedPath};
pub use crate::schedule::Schedule;
pub use crate::station::Station;
pub use crate::train::Train;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    pub use crate::dataset::fixtures as datasets;
    pub use crate::line::fixtures as lines;
    pub use crate::path::fixtures::{paths, segments};
    pub use crate::schedule::fixtures as schedules;
    pub use crate::station::fixtures as stations;
    pub use crate::train::fixtures as trains;
}
