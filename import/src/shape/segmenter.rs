use std::collections::HashMap;
use std::mem::replace;
use std::ops::Range;

use ordered_float::NotNan;

use super::{Buffer, Order, Segment, SegmentRef, SegmentedShape, ShapeId, Shapes};
use crate::coord::Point;

#[derive(Debug, PartialEq, Eq, Hash)]
struct HashablePoint(NotNan<f64>, NotNan<f64>);

impl From<Point> for HashablePoint {
    fn from(point: Point) -> Self {
        Self(NotNan::from(point.x), NotNan::from(point.y))
    }
}

#[derive(Debug, Clone)]
struct ReuseOpportunity {
    segment_index: usize,
    pos: usize,
}

impl ReuseOpportunity {
    fn can_start_reuse(&self, other: &Self) -> Option<Reuse> {
        if self.segment_index == other.segment_index {
            if self.pos + 1 == other.pos {
                return Some(Reuse {
                    segment_index: self.segment_index,
                    order: Order::Forward,
                    range: self.pos..other.pos + 1,
                });
            } else if self.pos == other.pos + 1 {
                return Some(Reuse {
                    segment_index: self.segment_index,
                    order: Order::Backward,
                    range: other.pos..self.pos + 1,
                });
            }
        }
        None
    }

    fn extend_reuse(&self, reuse: &mut Reuse) -> bool {
        if self.segment_index == reuse.segment_index {
            match reuse.order {
                Order::Forward if self.pos == reuse.range.end => {
                    reuse.range.end += 1;
                    true
                }
                Order::Backward if self.pos + 1 == reuse.range.start => {
                    reuse.range.start -= 1;
                    true
                }
                _ => false,
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Default)]
struct Accumulate {
    points: Vec<Point>,
    opportunities: Vec<ReuseOpportunity>,
}

#[derive(Debug)]
struct Reuse {
    segment_index: usize,
    order: Order,
    range: Range<usize>,
}

#[derive(Debug)]
enum State {
    Accumulate(Accumulate),
    Reuse(Reuse),
}

impl State {
    fn new() -> Self {
        Self::Accumulate(Accumulate::default())
    }

    fn process(&mut self, point: Point, opportunities: &[ReuseOpportunity]) -> Option<Self> {
        match self {
            Self::Accumulate(accumulate) => {
                for start_opportunity in accumulate.opportunities.iter() {
                    for opportunity in opportunities {
                        if let Some(reuse) = start_opportunity.can_start_reuse(opportunity) {
                            accumulate.points.pop();
                            let state = replace(self, Self::Reuse(reuse));
                            return Some(state);
                        }
                    }
                }
                accumulate.points.push(point);
                accumulate.opportunities = opportunities.to_vec();
                None
            }
            Self::Reuse(reuse) => {
                for opportunity in opportunities {
                    if opportunity.extend_reuse(reuse) {
                        return None;
                    }
                }
                let accumulate = Accumulate {
                    points: vec![point],
                    opportunities: opportunities.to_vec(),
                };
                let state = replace(self, Self::Accumulate(accumulate));
                Some(state)
            }
        }
    }

    fn refresh(&mut self, opportunities: &[ReuseOpportunity]) {
        match self {
            Self::Accumulate(accumulate) => accumulate.opportunities = opportunities.to_vec(),
            Self::Reuse(_) => {}
        }
    }

    fn apply(
        self,
        segmenter: &mut Segmenter,
        current_shape: &mut SegmentedShape,
    ) -> Option<SegmentRef> {
        match self {
            Self::Accumulate(Accumulate { points, .. }) => {
                if points.is_empty() {
                    None
                } else {
                    Some(segmenter.create_segment(points))
                }
            }
            Self::Reuse(reuse) => Some(segmenter.reuse_segment(reuse, current_shape)),
        }
    }

    fn apply_to(self, segmenter: &mut Segmenter, current_shape: &mut SegmentedShape) {
        if let Some(segment_ref) = self.apply(segmenter, current_shape) {
            current_shape.add(segment_ref);
        }
    }
}

pub(super) struct Segmenter {
    shapes: HashMap<ShapeId, SegmentedShape>,
    segments: Vec<Segment>,
    opportunities: HashMap<HashablePoint, Vec<ReuseOpportunity>>,
}

impl Segmenter {
    pub(super) fn new() -> Self {
        Segmenter {
            shapes: HashMap::new(),
            segments: Vec::new(),
            opportunities: HashMap::new(),
        }
    }

    fn create_segment(&mut self, points: Vec<Point>) -> SegmentRef {
        let segment_index = self.segments.len();
        for (pos, &point) in points.iter().enumerate() {
            self.opportunities
                .entry(point.into())
                .or_insert_with(Vec::new)
                .push(ReuseOpportunity { segment_index, pos });
        }
        self.segments.push(Segment::new(points));
        SegmentRef::new(segment_index, Order::Forward)
    }

    fn update_reuse_opportunities(
        &mut self,
        old_segment_index: usize,
        split_segment_index: usize,
        split_segment: impl Iterator<Item = Point>,
        difference: usize,
    ) {
        for (pos, point) in split_segment.enumerate() {
            let opportunity = self
                .opportunities
                .get_mut(&point.into())
                .unwrap()
                .iter_mut()
                .find(|opportunity| {
                    opportunity.segment_index == old_segment_index
                        && opportunity.pos == pos + difference
                })
                .unwrap();
            *opportunity = ReuseOpportunity {
                segment_index: split_segment_index,
                pos,
            };
        }
    }

    fn split_segment(&mut self, old_segment_index: usize, at: usize) -> usize {
        let split_segment_index = self.segments.len();
        let split_segment = self.segments[old_segment_index].split(at);
        self.update_reuse_opportunities(
            old_segment_index,
            split_segment_index,
            split_segment.iter(),
            at,
        );
        self.segments.push(split_segment);
        split_segment_index
    }

    fn reuse_segment(&mut self, reuse: Reuse, current_shape: &mut SegmentedShape) -> SegmentRef {
        let mut splits = Vec::new();
        let mut reused_segment_index = reuse.segment_index;
        if reuse.range.start > 0 {
            reused_segment_index = self.split_segment(reused_segment_index, reuse.range.start);
            splits.push(reused_segment_index);
        }
        let size = reuse.range.end - reuse.range.start;
        if size < self.segments[reused_segment_index].size() {
            let cutoff_segment_index = self.split_segment(reused_segment_index, size);
            splits.push(cutoff_segment_index);
        }

        if !splits.is_empty() {
            for shape in self.shapes.values_mut() {
                shape.apply_segment_split(reuse.segment_index, &splits);
            }
            current_shape.apply_segment_split(reuse.segment_index, &splits);
        }

        SegmentRef::new(reused_segment_index, reuse.order)
    }

    fn opportunities(&self, point: Point) -> &[ReuseOpportunity] {
        self.opportunities
            .get(&point.into())
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub(super) fn segment(&mut self, id: ShapeId, buffer: Buffer) {
        let mut current_shape = SegmentedShape::new();
        let mut state = State::new();
        for point in buffer {
            if let Some(task) = state.process(point, self.opportunities(point)) {
                task.apply_to(self, &mut current_shape);
                state.refresh(self.opportunities(point));
            }
        }
        state.apply_to(self, &mut current_shape);
        self.shapes.insert(id, current_shape);
    }

    pub(super) fn finish(self) -> Shapes {
        Shapes::new(self.shapes, self.segments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{segments, shape_buffers};
    use test_utils::{assert_eq_alternate, map};

    macro_rules! test_segmentation {
        ([
            $(
                $line:ident :: $shape:ident => [
                    $( $index:literal, $order:ident );* $(;)?
                ]
            ),* $(,)?
        ], [
            $( $segment:ident ),* $(,)?
        ]) => {{
            let mut segmenter = Segmenter::new();
            $(
                segmenter.segment(
                    format!("{}::{}", stringify!($line), stringify!($shape)).as_str().into(),
                    shape_buffers::$line::$shape(),
                );
            )*
            assert_eq_alternate!(
                segmenter.finish(),
                Shapes::new(
                    map! {
                        $(
                            format!("{}::{}", stringify!($line), stringify!($shape)).as_str() =>
                            vec![
                                $( SegmentRef::new($index, Order::$order) ),*
                            ].into()
                        ),*
                    },
                    vec![ $( segments::$segment() ),* ],
                )
            );
        }};
    }

    #[test]
    fn test_single_shape() {
        test_segmentation!(
            [
                u4::nollendorfplatz_innsbrucker_platz => [
                    0,  Forward;
                ],
            ],
            [
                nollendorfplatz_innsbrucker_platz,
            ]
        );
    }

    #[test]
    fn test_reversed_shape() {
        test_segmentation!(
            [
                u4::nollendorfplatz_innsbrucker_platz => [
                    0,  Forward;
                ],
                u4::innsbrucker_platz_nollendorfplatz => [
                    0, Backward;
                ],
            ],
            [
                nollendorfplatz_innsbrucker_platz,
            ]
        );
    }

    #[test]
    fn test_split_shape() {
        test_segmentation!(
            [
                tram_m10::clara_jaschke_str_warschauer_str => [
                    0,  Forward; 1,  Forward;
                ],
                tram_m10::clara_jaschke_str_landsberger_allee_petersburger_str => [
                    0,  Forward;
                ],
            ],
            [
                clara_jaschke_str_landsberger_allee_petersburger_str,
                landsberger_allee_petersburger_str_warschauer_str,
            ]
        );
    }

    #[test]
    fn test_reversed_shape_with_different_endpoint() {
        test_segmentation!(
            [
                tram_m10::clara_jaschke_str_landsberger_allee_petersburger_str => [
                    0,  Forward; 1,  Forward;
                ],
                tram_m10::landsberger_allee_petersburger_str_lueneburger_str => [
                    1, Backward; 2,  Forward;
                ],
            ],
            [
                clara_jaschke_str_hauptbahnhof,
                hauptbahnhof_landsberger_allee_petersburger_str,
                hauptbahnhof_lueneburger_str,
            ]
        );
    }

    #[test]
    fn test_different_endpoint_and_split() {
        test_segmentation!(
            [
                tram_m10::clara_jaschke_str_warschauer_str => [
                    0,  Forward; 3,  Forward; 5, Forward; 1, Forward;
                ],
                tram_m10::warschauer_str_lueneburger_str => [
                    1, Backward; 2,  Forward; 5, Backward; 3, Backward; 4,  Forward;
                ],
                tram_m10::clara_jaschke_str_landsberger_allee_petersburger_str => [
                    0,  Forward; 3,  Forward;
                ],
                tram_m10::landsberger_allee_petersburger_str_lueneburger_str => [
                    3, Backward; 4,  Forward;
                ],
            ],
            [
                clara_jaschke_str_hauptbahnhof,
                warschauer_str,
                revaler_str,
                hauptbahnhof_landsberger_allee_petersburger_str,
                hauptbahnhof_lueneburger_str,
                landsberger_allee_petersburger_str_revaler_str,
            ]
        );
    }

    #[test]
    fn test_crossing() {
        test_segmentation!(
            [
                s3::westkreuz_outbound => [
                    0,  Forward;
                ],
                s41::westkreuz_anticlockwise => [
                    1,  Forward;
                ],
            ],
            [
                westkreuz_stadtbahn,
                westkreuz_ringbahn,
            ]
        );
    }

    #[test]
    fn test_crossing_with_reversed() {
        test_segmentation!(
            [
                s3::westkreuz_outbound => [
                    0,  Forward;
                ],
                s41::westkreuz_anticlockwise => [
                    1,  Forward;
                ],
                s3::westkreuz_inbound => [
                    0, Backward;
                ],
                s42::westkreuz_clockwise => [
                    1, Backward;
                ],
            ],
            [
                westkreuz_stadtbahn,
                westkreuz_ringbahn,
            ]
        );
    }
}
