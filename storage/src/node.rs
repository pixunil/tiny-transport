use std::rc::Rc;

use na::Point2;
use serde_derive::{Deserialize, Serialize};

use simulation::Directions;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Node {
    position: Point2<f32>,
    kind: Kind,
    in_directions: Directions,
}

impl Node {
    pub fn new(position: Point2<f32>, kind: Kind, in_directions: Directions) -> Self {
        Self {
            position,
            kind,
            in_directions,
        }
    }

    pub(crate) fn station(&self) -> Option<usize> {
        match self.kind {
            Kind::Waypoint => None,
            Kind::Stop { at } => Some(at),
        }
    }

    pub fn load(self, stations: &[Rc<simulation::Station>]) -> simulation::Node {
        simulation::Node::new(self.position, self.kind.load(stations), self.in_directions)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Kind {
    Waypoint,
    Stop { at: usize },
}

impl Kind {
    fn load(self, stations: &[Rc<simulation::Station>]) -> simulation::NodeKind {
        match self {
            Self::Waypoint => simulation::NodeKind::Waypoint,
            Self::Stop { at } => simulation::NodeKind::Stop {
                at: stations[at].clone(),
            },
        }
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use std::ops::Index;

    use super::*;

    macro_rules! nodes {
        (@kind $station_ids:expr, $station:ident) => (
            Kind::Stop { at: $station_ids[stringify!($station)] }
        );
        (@kind) => ( Kind::Waypoint );
        ($($line:ident: $($x:literal, $y:literal, $in_directions:ident $(, $location:ident)?);* $(;)?)*) => (
            $(
                pub fn $line<'a>(station_ids: &impl Index<&'a str, Output = usize>) -> Vec<Node> {
                    vec![$(
                        Node {
                            position: Point2::new($x as f32, $y as f32),
                            kind: nodes!(@kind $( station_ids, $location )?),
                            in_directions: Directions::$in_directions,
                        }
                    ),*]
                }
            )*
        );
    }

    nodes! {
        s3:
               846,  -1428, Both,           hackescher_markt;
              -244,  -1229, Both,           friedrichstr;
             -1387,  -1700, Both,           hauptbahnhof;
             -2893,  -1178, Both,           bellevue;
        u6:
              -491,  -2348, Both,           naturkundemuseum;
              -164,  -1784, Both,           oranienburger_tor;
              -111,  -1115, Both,           friedrichstr;
               -55,   -558, Both,           franzoesische_str;
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
    }
}

#[cfg(test)]
mod tests {
    use crate::fixtures::nodes;
    use crate::fixtures_with_ids;

    #[test]
    fn test_load() {
        let (stations, station_ids) = fixtures_with_ids!(simulation::stations::{
            oranienburger_tor,
            friedrichstr,
            universitaetsstr,
            am_kupfergraben,
            georgenstr_am_kupfergraben,
        } with Rc);

        let nodes = nodes::tram_12(&station_ids);
        let expected = simulation::fixtures::nodes::tram_12();
        for (node, expected) in nodes.into_iter().zip(expected) {
            assert_eq!(node.load(&stations), expected)
        }
    }
}
