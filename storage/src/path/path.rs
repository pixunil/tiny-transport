use common::Order;

use crate::path::Node;
use crate::Segment;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SegmentedPath {
    segments: Vec<SegmentRef>,
}

impl SegmentedPath {
    pub fn new(segments: Vec<SegmentRef>) -> Self {
        Self { segments }
    }

    pub(crate) fn nodes(&self, segments: &[Segment]) -> impl Iterator<Item = Node> {
        let mut nodes = Vec::new();
        for segment in &self.segments {
            match segment.order {
                Order::Forward => {
                    nodes.extend(segments[segment.segment_index].nodes().iter().cloned());
                }
                Order::Backward => {
                    nodes.extend(
                        segments[segment.segment_index]
                            .nodes()
                            .iter()
                            .cloned()
                            .rev(),
                    );
                }
            }
        }
        nodes.into_iter()
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
                    use std::ops::Index;

                    use crate::path::{SegmentedPath, SegmentRef};
                    use common::{Order};

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
    }
}
