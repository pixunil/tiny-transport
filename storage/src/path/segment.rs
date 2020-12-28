use crate::path::Node;

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Segment {
    nodes: Vec<Node>,
}

impl Segment {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self { nodes }
    }

    pub(super) fn nodes(&self) -> &[Node] {
        &self.nodes
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use std::ops::Index;

    use na::Point2;

    use super::*;
    use crate::path::node::Kind;

    macro_rules! segments {
        (@kind $station_ids:expr, $station:ident) => (
            Kind::Stop {at: $station_ids[stringify!($station)]}
        );
        (@kind) => ( Kind::Waypoint );
        ($( $segment:ident : [ $( $x:literal, $y:literal $( , $station:ident )? );* $(;)? ] ),* $(,)?) => {
            $(
                pub fn $segment<'a>(station_ids: &impl Index<&'a str, Output = usize>) -> Segment {
                    Segment {
                        nodes: vec![ $(
                            Node::new(
                                Point2::new($x as f32, $y as f32),
                                segments!(@kind $(station_ids, $station)?),
                            )
                        ),* ],
                    }
                }
            )*
        }
    }

    segments! {
        hackescher_markt_bellevue: [
               846,  -1428, hackescher_markt;
              -244,  -1229, friedrichstr;
             -1387,  -1700, hauptbahnhof;
             -2893,  -1178, bellevue;
        ],
        naturkundemuseum_franzoesische_str: [
              -491,  -2348, naturkundemuseum;
              -164,  -1784, oranienburger_tor;
              -111,  -1115, friedrichstr;
               -55,   -558, franzoesische_str;
        ],
        oranienburger_tor_friedrichstr: [
               -98,  -1671, oranienburger_tor;
              -101,  -1560;
              -108,  -1226;
              -111,  -1115, friedrichstr;
               -46,  -1003;
                22,  -1001;
        ],
        universitaetsstr_am_kupfergraben: [
                90,   -999;
               158,   -998, universitaetsstr;
               429,   -992, am_kupfergraben;
        ],
        am_kupfergraben_georgenstr: [
               429,   -992, am_kupfergraben;
               432,  -1103;
               367,  -1216;
               299,  -1217;
               228,  -1108, georgenstr_am_kupfergraben;
                93,  -1111;
                25,  -1112;
        ],
    }
}
