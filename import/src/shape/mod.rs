mod importer;
mod record;
mod segment;
mod segmented_shape;
mod segmenter;
mod shape;
mod smoother;

#[cfg(test)]
pub(crate) mod fixtures {
    pub(crate) use super::segment::fixtures as segments;
    pub(crate) use super::shape::fixtures as shapes;
}

use record::ShapeRecord;
use segment::Segment;
use segmented_shape::{Order, SegmentRef, SegmentedShape};
use segmenter::Segmenter;

pub(crate) use importer::Importer;
pub(crate) use shape::{Shape, ShapeId};
pub use smoother::Mode as SmoothMode;

use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub(crate) struct Shapes {
    shapes: HashMap<ShapeId, SegmentedShape>,
    segments: Vec<Segment>,
}

impl Shapes {
    pub(super) fn new(shapes: HashMap<ShapeId, SegmentedShape>, segments: Vec<Segment>) -> Self {
        Self { shapes, segments }
    }

    pub(crate) fn glue_together(self) -> HashMap<ShapeId, Shape> {
        let segments = &self.segments;
        self.shapes
            .into_iter()
            .map(|(id, shape)| (id, shape.glue(segments)))
            .collect()
    }
}
