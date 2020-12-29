use crate::path::Node;

#[derive(Debug, PartialEq)]
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
    use std::rc::Rc;

    use na::Point2;

    use super::*;
    use crate::fixtures::stations;
    use crate::path::node::Kind;

    macro_rules! segments {
        (@kind $station:ident) => (
            Kind::Stop { at: Rc::new(stations::$station()) }
        );
        (@kind) => ( Kind::Waypoint );
        ($( $segment:ident : [ $( $x:literal, $y:literal $( , $station:ident )? );* $(;)? ] ),* $(,)?) => {
            $(
                pub fn $segment<'a>() -> Segment {
                    Segment {
                        nodes: vec![ $(
                            Node::new(
                                Point2::new($x as f32, $y as f32),
                                segments!(@kind $($station)?),
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
        zingster_str: [
              7204,  -6855;
              7269,  -6742, zingster_str;
        ],
        zingster_str_ribnitzer_str: [
              7335,  -6629;
              7337,  -6741;
        ],
        zingster_str_ribnitzer_str_prerower_platz: [
              7400,  -6517, zingster_str_ribnitzer_str;
              7662,  -6066, ahrenshooper_str;
              7795,  -5952;
              7926,  -5727, prerower_platz;
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
        weskammstr_waldsassener_str: [
             -2958,  10616, weskammstr;
             -2958,  10616;
             -2961,  10727;
             -2963,  10838, lichterfelder_ring_waldsassener_str;
             -2966,  10949;
             -2968,  11061;
             -2903,  11173;
             -2906,  11285, waldsassener_str;
             -2906,  11285;
             -2906,  11285;
        ],
    }
}
