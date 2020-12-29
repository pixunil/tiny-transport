use common::Order;

use crate::path::{Node, Segment};

#[derive(Debug, PartialEq, Eq)]
pub struct SegmentRef {
    segment_index: usize,
    order: Order,
}

impl SegmentRef {
    pub fn new(segment_index: usize, order: Order) -> Self {
        Self {
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
    pub fn new(segments: Vec<SegmentRef>) -> Self {
        Self { segments }
    }

    pub fn nodes<'a>(&self, segments: &'a [Segment]) -> Vec<&'a Node> {
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
        nodes
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
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
                pub mod $line {
                    use std::collections::HashMap;
                    use std::ops::Index;

                    use crate::path::{SegmentedPath, SegmentRef, Segment};
                    use common::{fixtures_with_ids, Order};

                    $(
                        pub fn $path<'a>(
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
                    pub fn segments() -> (Vec<Segment>, HashMap<&'static str, usize>) {
                        fixtures_with_ids!(segments::{ $( $( $segment, )* )* })
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
        u6: {
            naturkundemuseum_franzoesische_str: [
                naturkundemuseum_franzoesische_str,                         Forward;
            ],
        },
        tram_m5: {
            zingster_str_prerower_platz: [
                zingster_str,                                               Forward;
                zingster_str_ribnitzer_str_prerower_platz,                  Forward;
            ],
            prerower_platz_zingster_str: [
                zingster_str_ribnitzer_str_prerower_platz,                  Backward;
                zingster_str_ribnitzer_str,                                 Forward;
                zingster_str,                                               Backward;
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
        bus_m82: {
            weskammstr_waldsassener_str: [
                weskammstr_waldsassener_str,                                Forward;
            ],
            waldsassener_str_weskammstr: [
                weskammstr_waldsassener_str,                                Backward;
            ],
        },
    }
}
