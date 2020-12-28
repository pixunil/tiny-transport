mod node;
mod path;
mod placer;
mod segment;

#[cfg(test)]
pub(crate) mod fixtures {
    pub(crate) use super::path::fixtures as paths;
    pub(crate) use super::segment::fixtures as path_segments;
}

use path::SegmentRef;

pub(crate) use node::Node;
pub(crate) use path::SegmentedPath;
pub(crate) use placer::StopPlacer;
pub use segment::Segment;
