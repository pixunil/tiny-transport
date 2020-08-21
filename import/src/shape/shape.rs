use std::iter;

use crate::coord::Point;
use crate::shape::Segment;

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

    pub(super) fn bind<'a>(&'a self, segments: &'a [Segment]) -> Shape<'a> {
        Shape {
            segments,
            segmented_shape: self,
        }
    }
}

#[cfg(test)]
impl From<Vec<SegmentRef>> for SegmentedShape {
    fn from(segments: Vec<SegmentRef>) -> Self {
        Self { segments }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct Shape<'a> {
    segments: &'a [Segment],
    segmented_shape: &'a SegmentedShape,
}

impl<'a> Shape<'a> {
    pub(crate) fn iter_count(&self, count: usize) -> impl Iterator<Item = Point> + '_ {
        let mut points = Vec::new();
        for segment_ref in &self.segmented_shape.segments {
            match segment_ref.order {
                Order::Forward => points.extend(self.segments[segment_ref.segment_index].iter()),
                Order::Backward => {
                    points.extend(self.segments[segment_ref.segment_index].iter().rev())
                }
            }
        }

        points.extend(
            iter::repeat(points.last().unwrap().clone()).take(count.saturating_sub(points.len())),
        );

        points.into_iter()
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    macro_rules! shapes {
        ($(
            $line:ident : {
                $(
                    $shape:ident : [ $(
                        $segment:ident, $order:ident
                    );* $(;)? ]
                ),* $(,)?
            }
        ),* $(,)?) => {
            use crate::shape::Shapes;
            use test_utils::{fixtures_with_ids, join, map};

            $(
                pub(crate) mod $line {
                    use std::ops::Index;

                    use crate::shape::{SegmentedShape, SegmentRef, Order, Shapes};
                    use test_utils::{fixtures_with_ids, join, map};

                    $(
                        pub(crate) fn $shape<'a>(
                            segments: &impl Index<&'a str, Output = usize>,
                        ) -> SegmentedShape {
                            SegmentedShape {
                                segments: vec![ $(
                                    SegmentRef::new(segments[stringify!($segment)], Order::$order)
                                ),* ],
                            }
                        }
                    )*

                    #[allow(dead_code)]
                    pub(crate) fn by_id() -> Shapes {
                        let (segments, segment_ids) =
                            fixtures_with_ids!(segments::{ $( $( $segment, )* )* });
                        let segmented_shapes = map!{
                            $(
                                join!($line, $shape) => $shape(&segment_ids),
                            )*
                        };
                        Shapes::new(segmented_shapes, segments)
                    }
                }
            )*

            pub(crate) fn by_id() -> Shapes {
                let (segments, segment_ids) =
                    fixtures_with_ids!(segments::{ $( $( $( $segment, )* )* )* });
                let segmented_shapes = map!{
                    $(
                        $(
                            join!($line, $shape) => $line::$shape(&segment_ids),
                        )*
                    )*
                };
                Shapes::new(segmented_shapes, segments)
            }
        };
    }

    shapes! {
        s41: {
            circle: [
                ringbahn,                                                   Forward;
            ],
        },
        u4: {
            nollendorfplatz_innsbrucker_platz: [
                nollendorfplatz_innsbrucker_platz,                          Forward;
            ],
            innsbrucker_platz_nollendorfplatz: [
                nollendorfplatz_innsbrucker_platz,                          Backward;
            ],
        },
        tram_m10: {
            clara_jaschke_str_warschauer_str: [
                clara_jaschke_str_hauptbahnhof,                             Forward;
                hauptbahnhof_landsberger_allee_petersburger_str,            Forward;
                landsberger_allee_petersburger_str_revaler_str,             Forward;
                warschauer_str,                                             Forward;
            ],
            warschauer_str_lueneburger_str: [
                warschauer_str,                                             Backward;
                revaler_str,                                                Forward;
                landsberger_allee_petersburger_str_revaler_str,             Backward;
                hauptbahnhof_landsberger_allee_petersburger_str,            Backward;
                hauptbahnhof_lueneburger_str,                               Forward;
            ],
            clara_jaschke_str_landsberger_allee_petersburger_str: [
                clara_jaschke_str_hauptbahnhof,                             Forward;
                hauptbahnhof_landsberger_allee_petersburger_str,            Forward;
            ],
            landsberger_allee_petersburger_str_lueneburger_str: [
                hauptbahnhof_landsberger_allee_petersburger_str,            Backward;
                hauptbahnhof_lueneburger_str,                               Forward;
            ],
        },
        tram_12: {
            oranienburger_tor_am_kupfergraben: [
                oranienburger_tor_friedrichstr,                             Forward;
                universitaetsstr_am_kupfergraben,                           Forward;
            ],
            am_kupfergraben_oranienburger_tor: [
                georgenstr_am_kupfergraben,                                 Forward;
                oranienburger_tor_friedrichstr,                             Backward;
            ],
        },
        bus_m41: {
            anhalter_bahnhof_hauptbahnhof: [
                anhalter_bahnhof_tiergarten,                                Forward;
                tiergarten_hauptbahnhof,                                    Forward;
            ],
            hauptbahnhof_anhalter_bahnhof: [
                hauptbahnhof_tiergarten,                                    Forward;
                anhalter_bahnhof_tiergarten,                                Backward;
            ],
        },
        bus_114: {
            wannsee_heckeshorn_wannsee: [
                wannsee_heckeshorn_wannsee,                                 Forward;
            ],
        },
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
