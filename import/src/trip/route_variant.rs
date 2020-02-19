use std::iter;
use std::rc::Rc;

use simulation::Direction;
use crate::shape::Shape;
use crate::location::Location;
use super::{Node, Trip, Route};

#[derive(Debug, PartialEq)]
pub(super) struct RouteVariant {
    pub(super) locations: Vec<Rc<Location>>,
    pub(super) shape: Shape,
    pub(super) trips: Vec<Trip>,
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

    pub(super) fn add_trip(&mut self, trip: Trip) {
        self.trips.push(trip);
    }

    fn nodes(&self, direction: Direction) -> Vec<Node> {
        let mut nodes = self.shape.iter()
            .chain(iter::repeat(self.shape.last().unwrap())
                .take(self.locations.len().checked_sub(self.shape.len()).unwrap_or(0)))
            .map(|waypoint| {
                Node::new(waypoint.clone(), direction.into())
            })
            .collect::<Vec<_>>();

        let mut lower = 0;
        for (i, location) in self.locations.iter().enumerate() {
            let upper = nodes.len() - (self.locations.len() - i);
            let pos = Self::find_nearest_node(&mut nodes[lower ..= upper], Rc::clone(location));
            lower += pos + 1;
        }

        nodes
    }

    fn find_nearest_node(nodes: &mut [Node], location: Rc<Location>) -> usize {
        let (pos, node) = nodes.iter_mut()
            .enumerate()
            .min_by_key(|(_, node)| node.distance_to(&location))
            .unwrap();
        node.make_stop(location);
        pos
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
            let merge_candidate = downstream_nodes.iter()
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
        ($($line:ident: {$($variant:ident: $route:ident, [$($hour:expr, $minute:expr);* $(;)?]),* $(,)?}),* $(,)?) => (
            $(
                pub(in crate::trip) mod $line {
                    use crate::trip::fixtures::*;
                    use crate::trip::route_variant::*;

                    $(
                        pub(in crate::trip) fn $variant() -> RouteVariant {
                            RouteVariant {
                                locations: stop_locations::$line::$route(),
                                shape: shapes::$line::$route(),
                                trips: vec![$( trips::$line::$route($hour, $minute) ),*],
                            }
                        }
                    )*
                }
            )*
        );
    }

    route_variants! {
        tram_12: {
            upstream_1_trip: oranienburger_tor_am_kupfergraben, [9, 2.0],
            downstream_1_trip: am_kupfergraben_oranienburger_tor, [8, 34.0],
            upstream_2_trips: oranienburger_tor_am_kupfergraben, [9, 2.0; 9, 12.0],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_abs_diff_eq;

    use simulation::Directions;
    use crate::shape;
    use crate::trip::fixtures::*;

    #[test]
    fn test_nodes_upstream() {
        let variant = RouteVariant::new(stop_locations::tram_12::oranienburger_tor_am_kupfergraben(), shapes::tram_12::oranienburger_tor_am_kupfergraben());
        assert_eq!(variant.nodes(Direction::Upstream), nodes::tram_12(Directions::UpstreamOnly));
    }

    #[test]
    fn test_nodes_downstream() {
        let variant = RouteVariant::new(stop_locations::tram_12::am_kupfergraben_oranienburger_tor(), shapes::tram_12::am_kupfergraben_oranienburger_tor());
        let mut expected_nodes = nodes::tram_12(Directions::DownstreamOnly);
        expected_nodes.reverse();
        assert_eq!(variant.nodes(Direction::Downstream), expected_nodes);
    }

    #[test]
    fn test_nodes_merging() {
        let upstream = RouteVariant::new(stop_locations::tram_12::oranienburger_tor_am_kupfergraben(), shapes::tram_12::oranienburger_tor_am_kupfergraben());
        let downstream = RouteVariant::new(stop_locations::tram_12::am_kupfergraben_oranienburger_tor(), shapes::tram_12::am_kupfergraben_oranienburger_tor());
        assert_eq!(upstream.merge_nodes(&downstream), nodes::tram_12(Directions::Both));
    }

    #[test]
    fn test_nodes_circle() {
        let shape = shape!(52.549, 13.388; 52.503, 13.469; 52.475, 13.366; 52.501, 13.283; 52.549, 13.388);
        let locations = vec![Rc::new(locations::gesundbrunnen()),
                             Rc::new(locations::ostkreuz()), Rc::new(locations::suedkreuz()),
                             Rc::new(locations::westkreuz()), Rc::new(locations::gesundbrunnen())];
        let variant = RouteVariant::new(locations, shape);
        assert_abs_diff_eq!(*variant.nodes(Direction::Upstream), nodes::circle(Directions::UpstreamOnly), epsilon = 0.01);
    }
}
