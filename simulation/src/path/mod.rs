mod node;
mod path;
mod segment;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    pub use super::path::fixtures as paths;
    pub use super::segment::fixtures as segments;
}

pub use node::{Kind as NodeKind, Node};
pub use path::{SegmentRef, SegmentedPath};
pub use segment::Segment;
