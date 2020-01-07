use serde_derive::{Serialize, Deserialize};

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
        self.kind == other.kind &&
            Point2::abs_diff_eq(&self.position, &other.position, epsilon)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Kind {
    Waypoint,
    Stop,
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;

    macro_rules! nodes {
        (kind Stop) => ( Kind::Stop );
        (kind) => ( Kind::Waypoint );
        ($($line:ident: $($x:literal, $y:literal, $in_directions:ident $(, $location:ident)?);* $(;)?)*) => (
            $(
                pub(crate) fn $line() -> Vec<Node> {
                    vec![$(
                        Node {
                            position: Point2::new($x, $y),
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
            200.0, 100.0, Both, Stop;
            220.0, 100.0, Both;
            230.0, 105.0, Both, Stop;
    }
}
