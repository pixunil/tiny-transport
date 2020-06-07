use std::rc::Rc;

use na::Point2;

use crate::direction::{Direction, Directions};
use crate::station::Station;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    position: Point2<f32>,
    kind: Kind,
    in_directions: Directions,
}

impl Node {
    pub fn new(position: Point2<f32>, kind: Kind, in_directions: Directions) -> Node {
        Node {
            position,
            kind,
            in_directions,
        }
    }

    pub fn position(&self) -> Point2<f32> {
        self.position
    }

    pub fn is_stop(&self) -> bool {
        match self.kind {
            Kind::Waypoint => false,
            Kind::Stop { .. } => true,
        }
    }

    pub fn allows(&self, direction: Direction) -> bool {
        self.in_directions.allows(direction)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Waypoint,
    Stop { at: Rc<Station> },
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use super::*;
    use crate::fixtures::stations;

    macro_rules! nodes {
        (kind $station:ident) => ( Kind::Stop { at: Rc::new(stations::$station()) } );
        (kind) => ( Kind::Waypoint );
        ($($line:ident: $($x:literal, $y:literal, $in_directions:ident $(, $location:ident)?);* $(;)?)*) => (
            $(
                pub fn $line() -> Vec<Node> {
                    vec![$(
                        Node {
                            position: Point2::new($x as f32, $y as f32),
                            kind: nodes!(kind $($location)?),
                            in_directions: Directions::$in_directions,
                        }
                    ),*]
                }
            )*
        );
    }

    nodes! {
        tram_m5:
              7204,  -6855, Both;
              7269,  -6742, Both,           zingster_str;
              7337,  -6741, DownstreamOnly;
              7335,  -6629, DownstreamOnly;
              7400,  -6517, Both,           zingster_str_ribnitzer_str;
              7662,  -6066, Both,           ahrenshooper_str;
              7795,  -5952, Both;
              7926,  -5727, Both,           prerower_platz;
        tram_12:
               -98,  -1671, Both,           oranienburger_tor;
              -101,  -1560, Both;
              -106,  -1338, DownstreamOnly;
              -108,  -1226, Both;
              -111,  -1115, Both,           friedrichstr;
              -113,  -1004, UpstreamOnly;
               -46,  -1003, Both;
                22,  -1001, Both;
                90,   -999, UpstreamOnly;
               158,   -998, UpstreamOnly,   universitaetsstr;
                25,  -1112, DownstreamOnly;
                93,  -1111, DownstreamOnly;
               228,  -1108, DownstreamOnly, georgenstr_am_kupfergraben;
               299,  -1217, DownstreamOnly;
               367,  -1216, DownstreamOnly;
               432,  -1103, DownstreamOnly;
               429,   -992, Both,           am_kupfergraben;
        bus_m82:
             -2958,  10616, Both,           weskammstr;
             -2958,  10616, Both;
             -2961,  10727, Both;
             -2963,  10838, Both,           lichterfelder_ring_waldsassener_str;
             -2966,  10949, Both;
             -2968,  11061, Both;
             -2900,  11062, DownstreamOnly;
             -2903,  11173, Both;
             -2906,  11285, Both,           waldsassener_str;
             -2976,  11394, UpstreamOnly;
             -2906,  11285, Both;
             -2906,  11285, Both;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::nodes;

    #[test]
    fn test_getters() {
        let node = nodes::tram_12().remove(4);
        assert_eq!(node.position(), Point2::new(-111.0, -1115.0));
        assert!(node.is_stop());
        assert!(node.allows(Direction::Upstream));
        assert!(node.allows(Direction::Downstream));
    }
}
