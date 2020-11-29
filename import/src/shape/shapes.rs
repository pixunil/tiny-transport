use std::collections::HashMap;
use std::ops::Index;

use crate::shape::{Segment, SegmentedShape, ShapeId};

#[derive(Debug, PartialEq)]
pub(crate) struct Shapes {
    shapes: HashMap<ShapeId, SegmentedShape>,
    segments: Vec<Segment>,
}

impl Shapes {
    pub(super) fn new(shapes: HashMap<ShapeId, SegmentedShape>, segments: Vec<Segment>) -> Self {
        Self { shapes, segments }
    }

    pub(crate) fn segments(&self) -> &[Segment] {
        &self.segments
    }

    pub(super) fn segment_count(&self) -> usize {
        self.segments.len()
    }
}

impl<'a> Index<&'a ShapeId> for Shapes {
    type Output = SegmentedShape;

    fn index(&self, id: &'a ShapeId) -> &Self::Output {
        &self.shapes[id]
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
