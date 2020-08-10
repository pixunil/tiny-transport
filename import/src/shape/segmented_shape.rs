use crate::shape::{Segment, Shape};

use itertools::Itertools;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(super) enum Order {
    Forward,
    Backward,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct SegmentRef {
    segment_index: usize,
    order: Order,
}

impl SegmentRef {
    pub(super) fn new(segment_index: usize, order: Order) -> Self {
        Self {
            segment_index,
            order,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct SegmentedShape {
    segments: Vec<SegmentRef>,
}

impl SegmentedShape {
    pub(super) fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub(super) fn add(&mut self, segment_ref: SegmentRef) {
        self.segments.push(segment_ref);
    }

    pub(super) fn apply_segment_split(&mut self, segment_index: usize, splits: &[usize]) {
        let mut inserted = 0;
        let positions = self
            .segments
            .iter()
            .positions(|segment_ref| segment_ref.segment_index == segment_index)
            .collect::<Vec<_>>();
        for position in positions {
            let pos = position + inserted;
            match self.segments[pos].order {
                Order::Forward => {
                    self.segments.splice(
                        pos + 1..pos + 1,
                        splits
                            .iter()
                            .map(|&segment_index| SegmentRef::new(segment_index, Order::Forward)),
                    );
                }
                Order::Backward => {
                    self.segments.splice(
                        pos..pos,
                        splits
                            .iter()
                            .rev()
                            .map(|&segment_index| SegmentRef::new(segment_index, Order::Backward)),
                    );
                }
            }
            inserted += splits.len();
        }
    }

    pub(super) fn glue(&self, segments: &[Segment]) -> Shape {
        let mut shape = Vec::new();
        for segment_ref in &self.segments {
            match segment_ref.order {
                Order::Forward => shape.extend(segments[segment_ref.segment_index].iter()),
                Order::Backward => shape.extend(segments[segment_ref.segment_index].iter().rev()),
            }
        }
        Shape::from(shape)
    }
}

#[cfg(test)]
impl From<Vec<SegmentRef>> for SegmentedShape {
    fn from(segments: Vec<SegmentRef>) -> Self {
        Self { segments }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::assert_eq_alternate;

    #[test]
    fn test_segment_split_forward() {
        let mut shape = SegmentedShape {
            segments: vec![SegmentRef::new(0, Order::Forward)],
        };
        shape.apply_segment_split(0, &[1, 2]);
        assert_eq_alternate!(
            shape,
            SegmentedShape {
                segments: vec![
                    SegmentRef::new(0, Order::Forward),
                    SegmentRef::new(1, Order::Forward),
                    SegmentRef::new(2, Order::Forward),
                ],
            }
        );
    }

    #[test]
    fn test_segment_split_backward() {
        let mut shape = SegmentedShape {
            segments: vec![SegmentRef::new(0, Order::Backward)],
        };
        shape.apply_segment_split(0, &[1, 2]);
        assert_eq_alternate!(
            shape,
            SegmentedShape {
                segments: vec![
                    SegmentRef::new(2, Order::Backward),
                    SegmentRef::new(1, Order::Backward),
                    SegmentRef::new(0, Order::Backward),
                ],
            }
        );
    }

    #[test]
    fn test_segment_split_multiple() {
        let mut shape = SegmentedShape {
            segments: vec![
                SegmentRef::new(0, Order::Forward),
                SegmentRef::new(0, Order::Forward),
            ],
        };
        shape.apply_segment_split(0, &[1]);
        assert_eq_alternate!(
            shape,
            SegmentedShape {
                segments: vec![
                    SegmentRef::new(0, Order::Forward),
                    SegmentRef::new(1, Order::Forward),
                    SegmentRef::new(0, Order::Forward),
                    SegmentRef::new(1, Order::Forward),
                ]
            }
        );
    }
}
