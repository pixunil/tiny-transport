use itertools::Itertools;

use crate::path::Node;
use crate::path::Segment;
use common::Order;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(super) struct SegmentRef {
    segment_index: usize,
    order: Order,
}

impl SegmentRef {
    pub(super) fn new(segment_index: usize, order: Order) -> Self {
        SegmentRef {
            segment_index,
            order,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SegmentedPath {
    segments: Vec<SegmentRef>,
}

impl SegmentedPath {
    pub(super) fn new(segments: Vec<SegmentRef>) -> Self {
        SegmentedPath { segments }
    }

    pub(crate) fn segment_weights(&self, segments: &[Segment]) -> Vec<f64> {
        self.nodes(segments)
            .filter(|node| node.location().is_some())
            .tuple_windows()
            .map(|(before, after)| na::distance(&before.position(), &after.position()))
            .collect()
    }

    pub fn nodes<'a>(&self, segments: &'a [Segment]) -> impl Iterator<Item = &'a Node> {
        let mut nodes = Vec::new();
        for segment in &self.segments {
            match segment.order {
                Order::Forward => {
                    nodes.extend(segments[segment.segment_index].nodes().iter());
                }
                Order::Backward => {
                    nodes.extend(segments[segment.segment_index].nodes().iter().rev());
                }
            }
        }
        nodes.into_iter()
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    macro_rules! paths {
        ($(
            $line:ident : {
                $(
                    $path:ident : [ $(
                        $segment:ident, $order:ident
                    );* $(;)? ]
                ),* $(,)?
            }
        ),* $(,)?) => {
            $(
                pub(crate) mod $line {
                    use std::collections::HashMap;
                    use std::ops::Index;

                    use crate::path::{SegmentedPath, SegmentRef, Segment};
                    use common::{fixtures_with_ids, Order};

                    $(
                        pub(crate) fn $path<'a>(
                            segments: &impl Index<&'a str, Output = usize>,
                        ) -> SegmentedPath {
                            SegmentedPath {
                                segments: vec![ $(
                                    SegmentRef::new(segments[stringify!($segment)], Order::$order)
                                ),* ],
                            }
                        }
                    )*

                    #[allow(dead_code)]
                    pub(crate) fn segments() -> (Vec<Segment>, HashMap<&'static str, usize>) {
                        fixtures_with_ids!(path_segments::{ $( $( $segment, )* )* })
                    }
                }
            )*
        };
    }

    paths! {
        s3: {
            hackescher_markt_bellevue: [
                hackescher_markt_bellevue,                                  Forward;
            ],
        },
        s41: {
            circle: [
                circle,                                                     Forward;
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
            strassmannstr_warschauer_str_too_few_points: [
                strassmannstr_warschauer_str_too_few_points,                Forward;
            ],
        },
        tram_12: {
            oranienburger_tor_am_kupfergraben: [
                oranienburger_tor_friedrichstr,                             Forward;
                universitaetsstr_am_kupfergraben,                           Forward;
            ],
            am_kupfergraben_oranienburger_tor: [
                am_kupfergraben_georgenstr,                                 Forward;
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
