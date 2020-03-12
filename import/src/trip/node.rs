use std::rc::Rc;
use std::fmt;

use ordered_float::NotNan;

use approx::AbsDiffEq;

use simulation::Directions;
use crate::coord::{Point, transform, debug_position};
use crate::location::Location;

#[derive(PartialEq)]
pub(super) struct Node {
    position: Point,
    kind: Kind,
    in_directions: Directions,
}

impl Node {
    pub(super) fn new(position: Point, in_directions: Directions) -> Self {
        Self {
            position,
            kind: Kind::Waypoint,
            in_directions,
        }
    }

    pub(super) fn distance_to(&self, location: &Location) -> NotNan<f64> {
        let distance = na::distance(&self.position, &location.position());
        NotNan::new(distance).unwrap()
    }

    pub(super) fn location(&self) -> Option<&Rc<Location>> {
        match &self.kind {
            Kind::Waypoint => None,
            Kind::Stop { location } => Some(&location),
        }
    }

    pub(super) fn make_stop(&mut self, location: Rc<Location>) {
        self.kind = Kind::Stop { location };
    }

    pub(super) fn can_be_merged(&self, other: &Self) -> bool {
        self.position == other.position && self.kind == other.kind &&
            self.in_directions == Directions::UpstreamOnly &&
            other.in_directions == Directions::DownstreamOnly
    }

    pub(super) fn merge(&mut self, other: Self) {
        assert!(self.can_be_merged(&other));
        self.in_directions = Directions::Both;
    }

    pub(super) fn freeze(&self) -> simulation::Node {
        let kind = match self.kind {
            Kind::Waypoint => simulation::NodeKind::Waypoint,
            Kind::Stop { .. } => simulation::NodeKind::Stop,
        };
        let position = transform(self.position);
        simulation::Node::new(position, kind, self.in_directions)
    }
}

type Epsilon = <Point as AbsDiffEq>::Epsilon;

impl AbsDiffEq for Node {
    type Epsilon = Epsilon;

    fn default_epsilon() -> Epsilon {
        Point::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Node, epsilon: Epsilon) -> bool {
        Point::abs_diff_eq(&self.position, &other.position, epsilon) &&
            self.kind == other.kind &&
            self.in_directions == other.in_directions

    }
}

impl fmt::Debug for Node {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let position = debug_position(self.position, formatter.alternate());
        match &self.kind {
            Kind::Waypoint => {
                formatter.debug_struct("Waypoint")
                    .field("position", &position)
                    .field("in_directions", &self.in_directions)
                    .finish()
            },
            Kind::Stop {location} => {
                formatter.debug_struct("Stop")
                    .field("position", &position)
                    .field("location", &location)
                    .field("in_directions", &self.in_directions)
                    .finish()
            },
        }
    }
}

#[derive(PartialEq)]
pub(crate) enum Kind {
    Waypoint,
    Stop {
        location: Rc<Location>,
    },
}

#[cfg(test)]
pub(super) mod fixtures {
    use super::*;
    use crate::coord::project;
    use crate::location::fixtures::*;

    macro_rules! nodes {
        (kind $location:ident) => ( Kind::Stop { location: Rc::new(locations::$location()) });
        (kind) => ( Kind::Waypoint );
        (node Both, $position:expr, $kind:expr, $in_directions:ident) => (
            Node {
                position: $position,
                kind: $kind,
                in_directions: Directions::$in_directions,
            }
        );
        (node UpstreamOnly, $position:expr, $kind:expr, DownstreamOnly) => ( None );
        (node DownstreamOnly, $position:expr, $kind:expr, UpstreamOnly) => ( None );
        (node $single_direction:ident, $position:expr, $kind:expr, $in_directions:ident) => (
            Some(Node {
                position: $position,
                kind: $kind,
                in_directions: Directions::$single_direction,
            })
        );
        ($($line:ident: $($lat:literal, $lon:literal, $in_directions:ident $(, $location:ident)?);* $(;)?)*) => (
            $(
                pub(in crate::trip) fn $line(directions: Directions) -> Vec<Node> {
                    match directions {
                        Directions::Both => vec![$(
                            nodes!(node Both, project($lat, $lon), nodes!(kind $($location)?), $in_directions)
                        ),*],
                        Directions::UpstreamOnly => vec![$(
                            nodes!(node UpstreamOnly, project($lat, $lon), nodes!(kind $($location)?), $in_directions)
                        ),*].into_iter().filter_map(|node| node).collect(),
                        Directions::DownstreamOnly => vec![$(
                            nodes!(node DownstreamOnly, project($lat, $lon), nodes!(kind $($location)?), $in_directions)
                        ),*].into_iter().filter_map(|node| node).collect(),
                    }
                }
            )*
        );
    }

    nodes! {
        circle:
            52.549, 13.388, Both,           gesundbrunnen;
            52.503, 13.469, Both,           ostkreuz;
            52.475, 13.366, Both,           suedkreuz;
            52.501, 13.283, Both,           westkreuz;
            52.549, 13.388, Both,           gesundbrunnen;
        tram_12:
            52.525, 13.388, Both,           oranienburger_tor;
            52.524, 13.388, Both;
            52.522, 13.388, DownstreamOnly;
            52.521, 13.388, Both;
            52.520, 13.388, Both,           friedrichstr;
            52.519, 13.388, UpstreamOnly;
            52.519, 13.389, Both;
            52.519, 13.390, Both;
            52.519, 13.391, UpstreamOnly;
            52.519, 13.392, UpstreamOnly,   universitaetsstr;
            52.520, 13.390, DownstreamOnly;
            52.520, 13.391, DownstreamOnly;
            52.520, 13.393, DownstreamOnly, georgenstr_am_kupfergraben;
            52.521, 13.394, DownstreamOnly;
            52.521, 13.395, DownstreamOnly;
            52.520, 13.396, DownstreamOnly;
            52.519, 13.396, Both,           am_kupfergraben;
        bus_114:
            52.422, 13.178, UpstreamOnly;
            52.421, 13.178, UpstreamOnly, wannsee;
            52.421, 13.177, UpstreamOnly;
            52.421, 13.176, UpstreamOnly;
            52.420, 13.175, UpstreamOnly, wannseebruecke;
            52.420, 13.174, UpstreamOnly;
            52.421, 13.174, UpstreamOnly;
            52.421, 13.173, UpstreamOnly;
            52.421, 13.172, UpstreamOnly;
            52.421, 13.171, UpstreamOnly;
            52.421, 13.170, UpstreamOnly;
            52.421, 13.169, UpstreamOnly;
            52.421, 13.168, UpstreamOnly;
            52.421, 13.167, UpstreamOnly, am_kleinen_wannsee;
            52.421, 13.166, UpstreamOnly;
            52.421, 13.165, UpstreamOnly;
            52.422, 13.165, UpstreamOnly;
            52.422, 13.164, UpstreamOnly;
            52.423, 13.163, UpstreamOnly;
            52.423, 13.162, UpstreamOnly;
            52.424, 13.162, UpstreamOnly, seglerweg;
            52.425, 13.161, UpstreamOnly;
            52.426, 13.161, UpstreamOnly;
            52.427, 13.162, UpstreamOnly, koblanckstr;
            52.428, 13.162, UpstreamOnly;
            52.428, 13.163, UpstreamOnly;
            52.429, 13.164, UpstreamOnly, liebermann_villa;
            52.430, 13.164, UpstreamOnly;
            52.430, 13.165, UpstreamOnly;
            52.431, 13.165, UpstreamOnly;
            52.432, 13.165, UpstreamOnly, am_grossen_wannsee;
            52.433, 13.164, UpstreamOnly, haus_der_wannsee_konferenz;
            52.432, 13.163, UpstreamOnly;
            52.432, 13.162, UpstreamOnly;
            52.431, 13.162, UpstreamOnly;
            52.431, 13.161, UpstreamOnly;
            52.430, 13.161, UpstreamOnly, zum_heckeshorn;
            52.429, 13.160, UpstreamOnly;
            52.428, 13.160, UpstreamOnly;
            52.427, 13.160, UpstreamOnly, strasse_zum_loewen;
            52.427, 13.159, UpstreamOnly;
            52.426, 13.160, UpstreamOnly;
            52.424, 13.160, UpstreamOnly, seglerweg;
            52.421, 13.162, UpstreamOnly;
            52.420, 13.162, UpstreamOnly, conradstr;
            52.420, 13.166, UpstreamOnly;
            52.420, 13.167, UpstreamOnly, am_kleinen_wannsee;
            52.421, 13.168, UpstreamOnly;
            52.421, 13.170, UpstreamOnly;
            52.421, 13.171, UpstreamOnly;
            52.421, 13.172, UpstreamOnly;
            52.421, 13.173, UpstreamOnly;
            52.420, 13.174, UpstreamOnly;
            52.420, 13.175, UpstreamOnly, wannseebruecke;
            52.420, 13.176, UpstreamOnly;
            52.421, 13.176, UpstreamOnly;
            52.421, 13.177, UpstreamOnly;
            52.421, 13.178, UpstreamOnly, wannsee;
            52.422, 13.179, UpstreamOnly;
            52.422, 13.180, UpstreamOnly;
            52.422, 13.179, UpstreamOnly;
            52.422, 13.178, UpstreamOnly;
    }
}
