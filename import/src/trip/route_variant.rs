use std::iter;
use std::rc::Rc;

use ordered_float::NotNan;

use super::{Node, Route, Trip};
use crate::location::Location;
use crate::shape::Shape;
use itertools::Itertools;
use simulation::Direction;

struct StopCandidate {
    pos: usize,
    distance: NotNan<f64>,
    location: Rc<Location>,
}

impl StopCandidate {
    fn find_nearest(nodes: &[Node], lower: usize, upper: usize, location: Rc<Location>) -> Self {
        let (pos, node) = nodes[lower..upper]
            .iter()
            .enumerate()
            .min_by_key(|(_, node)| node.distance_to(&location))
            .unwrap();
        Self {
            pos: pos + lower,
            distance: node.distance_to(&location),
            location,
        }
    }

    fn distribute_across<'a>(nodes: &[Node], locations: &[Rc<Location>]) -> Vec<Self> {
        let mut candidates: Vec<Self> = Vec::with_capacity(locations.len());
        for (i, location) in locations.iter().enumerate() {
            let upper = nodes.len() + i - locations.len() + 1;
            let candidate_nearest = Self::find_nearest(&nodes, i, upper, Rc::clone(location));

            if candidates
                .last()
                .map_or(true, |last| last.pos < candidate_nearest.pos)
            {
                candidates.push(candidate_nearest);
                continue;
            }

            let (at, lower) = candidates
                .iter()
                .enumerate()
                .map(|(i, candidate)| (i + 1, candidate.pos + 1))
                .rfind(|&(at, lower)| {
                    let following = candidates.len() - at;
                    lower + following < candidate_nearest.pos
                })
                .unwrap_or((0, 0));
            let locations_brought_forward = candidates[at..]
                .iter()
                .map(|position| &position.location)
                .cloned()
                .collect::<Vec<_>>();
            let mut candidates_brought_forward = Self::distribute_across(
                &nodes[lower..candidate_nearest.pos],
                &locations_brought_forward,
            );
            for position in &mut candidates_brought_forward {
                position.pos += lower;
            }

            let candidate_behind = Self::find_nearest(
                &nodes,
                candidates.last().unwrap().pos + 1,
                upper,
                Rc::clone(location),
            );
            if candidate_nearest.total_difference(&candidates_brought_forward)
                <= candidate_behind.total_difference(&candidates[at..])
            {
                candidates.splice(at.., candidates_brought_forward);
                candidates.push(candidate_nearest);
            } else {
                candidates.push(candidate_behind);
            }
        }

        debug_assert!(candidates
            .iter()
            .tuple_windows()
            .all(|(a, b)| a.pos < b.pos));
        candidates
    }

    fn total_difference(&self, candidates: &[Self]) -> f64 {
        *self.distance
            + candidates
                .iter()
                .map(|candidate| *candidate.distance)
                .sum::<f64>()
    }

    fn accept(self, nodes: &mut [Node]) {
        nodes[self.pos].make_stop(self.location);
    }
}

#[derive(Debug, PartialEq)]
pub(super) struct RouteVariant {
    locations: Vec<Rc<Location>>,
    shape: Shape,
    trips: Vec<Trip>,
}

impl RouteVariant {
    pub(super) fn new(locations: Vec<Rc<Location>>, shape: Shape) -> Self {
        Self {
            locations,
            shape,
            trips: Vec::new(),
        }
    }

    pub(super) fn matches(&self, locations: &[Rc<Location>], shape: &Shape) -> bool {
        self.locations == locations && &self.shape == shape
    }

    pub(super) fn order(&self) -> impl Ord {
        self.trips.len()
    }

    pub(super) fn add_trip(&mut self, trip: Trip) {
        self.trips.push(trip);
    }

    fn nodes(&self, direction: Direction) -> Vec<Node> {
        let mut nodes = self
            .shape
            .iter()
            .chain(
                iter::repeat(self.shape.last().unwrap()).take(
                    self.locations
                        .len()
                        .checked_sub(self.shape.len())
                        .unwrap_or(0),
                ),
            )
            .map(|waypoint| Node::new(waypoint.clone(), direction.into()))
            .collect::<Vec<_>>();

        for candidate in StopCandidate::distribute_across(&nodes, &self.locations) {
            candidate.accept(&mut nodes);
        }

        nodes
    }

    pub(super) fn single(self, direction: Direction) -> Route {
        Route::new(self.nodes(direction), self.trips)
    }

    pub(super) fn merge(mut self, mut downstream: Self) -> Route {
        let nodes = self.merge_nodes(&downstream);
        self.trips.append(&mut downstream.trips);
        Route::new(nodes, self.trips)
    }

    fn merge_nodes(&self, downstream: &Self) -> Vec<Node> {
        let mut downstream_nodes = downstream.nodes(Direction::Downstream);
        let mut nodes = Vec::new();

        for mut node in self.nodes(Direction::Upstream) {
            let merge_candidate = downstream_nodes
                .iter()
                .rposition(|downstream| node.can_be_merged(&downstream));
            if let Some(pos) = merge_candidate {
                nodes.extend(downstream_nodes.splice(pos + 1.., iter::empty()).rev());
                node.merge(downstream_nodes.pop().unwrap());
            }

            nodes.push(node);
        }
        nodes.extend(downstream_nodes.into_iter().rev());
        nodes
    }
}

#[cfg(test)]
pub(super) mod fixtures {
    macro_rules! route_variants {
        ($(
            $line:ident: {
                $(
                    $variant:ident: $route:ident,
                    [ $( $( $(:)? $time:literal )* ),* $(,)?]
                ),* $(,)?
            }
        ),* $(,)?) => (
            $(
                pub(in crate::trip) mod $line {
                    use crate::trip::fixtures::*;
                    use crate::trip::route_variant::*;
                    use test_utils::time;

                    $(
                        pub(in crate::trip) fn $variant() -> RouteVariant {
                            RouteVariant {
                                locations: stop_locations::$line::$route(),
                                shape: shapes::$line::$route(),
                                trips: vec![$( trips::$line::$route(time!($($time),*)) ),*],
                            }
                        }
                    )*
                }
            )*
        );
    }

    route_variants! {
        tram_12: {
            upstream_1_trip: oranienburger_tor_am_kupfergraben, [9:02:00],
            downstream_1_trip: am_kupfergraben_oranienburger_tor, [8:34:00],
            upstream_2_trips: oranienburger_tor_am_kupfergraben, [9:02:00, 9:12:00],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape;
    use crate::trip::fixtures::*;
    use simulation::Directions;

    macro_rules! test_nodes {
        ($line:ident :: $route:ident, $direction:ident) => {{
            let variant =
                RouteVariant::new(stop_locations::$line::$route(), shapes::$line::$route());
            let mut expected_nodes = nodes::$line(Directions::from(Direction::$direction));
            if Direction::$direction == Direction::Downstream {
                expected_nodes.reverse();
            }
            assert_eq!(variant.nodes(Direction::$direction), expected_nodes);
            variant
        }};
        ($line:ident :: { $upstream:ident, $downstream:ident }) => {{
            let upstream = test_nodes!($line::$upstream, Upstream);
            let downstream = test_nodes!($line::$downstream, Downstream);
            assert_eq!(
                upstream.merge_nodes(&downstream),
                nodes::$line(Directions::Both)
            );
        }};
    }

    #[test]
    fn test_nodes_different_direction_stops() {
        test_nodes!(tram_12::{oranienburger_tor_am_kupfergraben, am_kupfergraben_oranienburger_tor});
    }

    #[test]
    fn test_nodes_circle() {
        let shape =
            shape!(52.549, 13.388; 52.503, 13.469; 52.475, 13.366; 52.501, 13.283; 52.549, 13.388);
        let locations = vec![
            Rc::new(locations::gesundbrunnen()),
            Rc::new(locations::ostkreuz()),
            Rc::new(locations::suedkreuz()),
            Rc::new(locations::westkreuz()),
            Rc::new(locations::gesundbrunnen()),
        ];
        let variant = RouteVariant::new(locations, shape);
        assert_eq!(
            variant.nodes(Direction::Upstream),
            nodes::circle(Directions::UpstreamOnly)
        );
    }

    #[test]
    fn test_nodes_duplicated_stop() {
        test_nodes!(bus_m41::{anhalter_bahnhof_hauptbahnhof, hauptbahnhof_anhalter_bahnhof});
    }

    #[test]
    fn test_nodes_lasso() {
        test_nodes!(bus_114::wannsee_heckeshorn_wannsee, Upstream);
    }
}
