use std::collections::HashMap;

use crate::shape::{Segment, SegmentedShape, Shape, ShapeId};

#[derive(Debug, PartialEq)]
pub(crate) struct Shapes {
    shapes: HashMap<ShapeId, SegmentedShape>,
    segments: Vec<Segment>,
}

impl Shapes {
    pub(super) fn new(shapes: HashMap<ShapeId, SegmentedShape>, segments: Vec<Segment>) -> Self {
        Self { shapes, segments }
    }

    pub(super) fn segment_count(&self) -> usize {
        self.segments.len()
    }

    pub(crate) fn bind(&self, id: &ShapeId) -> Shape {
        self.shapes[id].bind(&self.segments)
    }
}

#[cfg(test)]
mod tests {
    use crate::fixtures::shapes;

    #[test]
    fn test_segment_count() {
        let shapes = shapes::u4::by_id();
        assert_eq!(shapes.segment_count(), 1);
    }
}
