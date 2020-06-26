use std::iter;
use std::rc::Rc;

use ordered_float::NotNan;

use super::{Node, Route, Trip};
use crate::coord::Point;
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

    fn distribute_across(nodes: &[Node], locations: &[Rc<Location>]) -> Vec<Self> {
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

    pub(super) fn matches(&self, locations: &[Rc<Location>], shape: &[Point]) -> bool {
        self.locations == locations && self.shape == shape
    }

    pub(super) fn difference(&self, downstream: &Self) -> impl Ord {
        let mut sub_results = iter::repeat_with(|| {
            iter::repeat(0)
                .take(downstream.locations.len())
                .collect::<Vec<_>>()
        })
        .take(self.locations.len() + 1)
        .collect::<Vec<_>>();

        for (a, location_a) in self.locations.iter().enumerate() {
            for (b, location_b) in downstream.locations.iter().rev().enumerate() {
                if a == 0 || b == 0 {
                    sub_results[a][b] = a.max(b);
                    continue;
                }

                let mut option_match_or_replace = sub_results[a - 1][b - 1];
                if location_a != location_b {
                    option_match_or_replace += 1;
                }
                let option_add = sub_results[a - 1][b] + 1;
                let option_remove = sub_results[a][b - 1] + 1;
                sub_results[a][b] = option_match_or_replace.min(option_add).min(option_remove);
            }
        }

        sub_results[self.locations.len() - 1][downstream.locations.len() - 1]
    }

    pub(super) fn add_trip(&mut self, trip: Trip) {
        self.trips.push(trip);
    }

    fn nodes(&self, direction: Direction) -> Vec<Node> {
        let mut nodes = self
            .shape
            .iter()
            .chain(
                iter::repeat(self.shape.last().unwrap())
                    .take(self.locations.len().saturating_sub(self.shape.len())),
            )
            .map(|waypoint| Node::new(*waypoint, direction.into()))
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
pub(crate) mod fixtures {
    macro_rules! route_variants {
        (@trips $line:ident, $route:ident, []) => { vec![] };
        (@trips $line:ident, $route:ident, [$( $( $(:)? $time:literal )* ),* $(,)?]) => {{
            use crate::fixtures::trips;
            use test_utils::time;
            vec![ $( trips::$line::$route(time!($($time),*)) ),* ]
        }};
        ($(
            $line:ident: {
                $(
                    $name:ident: $route:ident, $times:tt
                ),* $(,)?
            }
        ),* $(,)?) => (
            $(
                pub(in crate::trip) mod $line {
                    use crate::fixtures::{shapes, stop_locations};
                    use crate::trip::route_variant::*;

                    $(
                        pub(in crate::trip) fn $name() -> RouteVariant {
                            RouteVariant {
                                locations: stop_locations::$line::$route(),
                                shape: shapes::$line::$route(),
                                trips: route_variants!(@trips $line, $route, $times),
                            }
                        }
                    )*
                }
            )*
        );
    }

    route_variants! {
        tram_m10: {
            clara_jaschke_str_warschauer_str:
                clara_jaschke_str_warschauer_str, [],
            warschauer_str_lueneburger_str:
                warschauer_str_lueneburger_str, [],
            clara_jaschke_str_landsberger_allee_petersburger_str:
                clara_jaschke_str_landsberger_allee_petersburger_str, [],
            landsberger_allee_petersburger_str_lueneburger_str:
                landsberger_allee_petersburger_str_lueneburger_str, [],
        },
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
    use crate::fixtures::{nodes, shapes, stop_locations};
    use simulation::Directions;

    macro_rules! test_nodes {
        ($line:ident :: $route:ident, $direction:ident) => {{
            test_nodes!($line::$route, $line::$route, $direction)
        }};
        ($line:ident :: $route:ident, $line_nodes:ident :: $nodes:ident, $direction:ident) => {{
            let variant =
                RouteVariant::new(stop_locations::$line::$route(), shapes::$line::$route());
            let directions = Directions::from(Direction::$direction);
            let mut expected_nodes = nodes::$line_nodes::$nodes(directions);
            if Direction::$direction == Direction::Downstream {
                expected_nodes.reverse();
            }
            assert_eq!(variant.nodes(Direction::$direction), expected_nodes);
            variant
        }};
        ($line:ident :: { $upstream:ident, $downstream:ident }) => {{
            let upstream = test_nodes!($line::$upstream, Upstream);
            let downstream = test_nodes!($line::$downstream, $line::$upstream, Downstream);
            assert_eq!(
                upstream.merge_nodes(&downstream),
                nodes::$line::$upstream(Directions::Both)
            );
        }};
    }

    #[test]
    fn test_nodes_different_direction_stops() {
        test_nodes!(tram_12::{oranienburger_tor_am_kupfergraben, am_kupfergraben_oranienburger_tor});
    }

    #[test]
    fn test_nodes_circle() {
        test_nodes!(s41::circle, Upstream);
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
