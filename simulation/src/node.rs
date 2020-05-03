use serde_derive::{Deserialize, Serialize};

use na::Point2;

use approx::AbsDiffEq;

use crate::direction::{Direction, Directions};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        self.kind == Kind::Stop
    }

    pub fn allows(&self, direction: Direction) -> bool {
        self.in_directions.allows(direction)
    }
}

type Epsilon = <Point2<f32> as AbsDiffEq>::Epsilon;

impl AbsDiffEq for Node {
    type Epsilon = Epsilon;

    fn default_epsilon() -> Epsilon {
        Point2::<f32>::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Node, epsilon: Epsilon) -> bool {
        self.kind == other.kind && Point2::abs_diff_eq(&self.position, &other.position, epsilon)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Kind {
    Waypoint,
    Stop,
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use super::*;

    macro_rules! nodes {
        (kind Stop) => ( Kind::Stop );
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
        blue:
               200,    100, Both,           Stop;
               220,    100, Both;
               230,    105, Both,           Stop;
        tram_12:
               -98,  -1671, Both,           Stop;
              -101,  -1560, Both;
              -106,  -1338, DownstreamOnly;
              -108,  -1226, Both;
              -111,  -1115, Both,           Stop;
              -113,  -1004, UpstreamOnly;
               -46,  -1003, Both;
                22,  -1001, Both;
                90,   -999, UpstreamOnly;
               158,   -998, UpstreamOnly,   Stop;
                25,  -1112, DownstreamOnly;
                93,  -1111, DownstreamOnly;
               228,  -1108, DownstreamOnly, Stop;
               299,  -1217, DownstreamOnly;
               367,  -1216, DownstreamOnly;
               432,  -1103, DownstreamOnly;
               429,   -992, Both,           Stop;
    }
}
