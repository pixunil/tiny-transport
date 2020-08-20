use std::rc::Rc;

use super::{Route, RouteVariant, Trip};
use crate::location::Location;
use crate::shape::Shape;
use simulation::Direction;

#[derive(Debug, PartialEq)]
pub(super) struct RouteBuffer<'a> {
    upstream: Vec<RouteVariant<'a>>,
    downstream: Vec<RouteVariant<'a>>,
}

impl<'a> RouteBuffer<'a> {
    pub(super) fn new() -> Self {
        Self {
            upstream: Vec::new(),
            downstream: Vec::new(),
        }
    }

    pub(super) fn add_trip(&mut self, locations: Vec<Rc<Location>>, shape: Shape<'a>, trip: Trip) {
        let variants = match trip.direction() {
            Direction::Upstream => &mut self.upstream,
            Direction::Downstream => &mut self.downstream,
        };
        let variant = variants
            .iter_mut()
            .find(|variant| variant.matches(&locations, &shape));

        match variant {
            Some(variant) => {
                variant.add_trip(trip);
            }
            None => {
                let mut variant = RouteVariant::new(locations, shape);
                variant.add_trip(trip);
                variants.push(variant);
            }
        }
    }

    pub(super) fn into_routes(mut self) -> Vec<Route> {
        let mut differences = self
            .upstream
            .iter()
            .map(|upstream| {
                self.downstream
                    .iter()
                    .map(|downstream| upstream.difference(downstream))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        fn find_pair(differences: &[Vec<impl Ord>]) -> Option<(usize, usize)> {
            differences
                .iter()
                .enumerate()
                .flat_map(|(a, sub_differences)| {
                    sub_differences
                        .iter()
                        .enumerate()
                        .map(move |(b, difference)| (a, b, difference))
                })
                .min_by_key(|(_, _, difference)| *difference)
                .map(|(a, b, _)| (a, b))
        }

        let mut routes = Vec::new();
        while let Some((a, b)) = find_pair(&differences) {
            routes.push(self.upstream.remove(a).merge(self.downstream.remove(b)));
            differences.remove(a);
            for sub_differences in &mut differences {
                sub_differences.remove(b);
            }
        }

        routes.extend(
            self.upstream
                .into_iter()
                .map(|variant| variant.single(Direction::Upstream)),
        );
        routes.extend(
            self.downstream
                .into_iter()
                .map(|variant| variant.single(Direction::Downstream)),
        );
        routes
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    macro_rules! route_buffers {
        ($($line:ident: {$($route:ident => [$($upstream:ident),* $(,)?], [$($downstream:ident),* $(,)?]),* $(,)?}),* $(,)?) => (
            $(
                pub(in crate::trip) mod $line {
                    use crate::fixtures::route_variants;
                    use crate::shape::Shapes;
                    use crate::trip::route_buffer::*;

                    $(
                        pub(in crate::trip) fn $route(shapes: &Shapes) -> RouteBuffer {
                            RouteBuffer {
                                upstream: vec![$(
                                    route_variants::$line::$upstream(&shapes)
                                ),*],
                                downstream: vec![$(
                                    route_variants::$line::$downstream(&shapes)
                                ),*],
                            }
                        }
                    )*
                }
            )*
        );
    }

    route_buffers! {
        tram_12: {
            with_1_upstream => [upstream_1_trip], [],
            with_1_downstream => [], [downstream_1_trip],
            with_1_upstream_1_downstream => [upstream_1_trip], [downstream_1_trip],
            with_2_upstream => [upstream_2_trips], [],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{route_buffers, route_variants, routes, shapes, stop_locations, trips};
    use test_utils::{assert_eq_alternate, time};

    #[test]
    fn test_create_first_upstream_variant() {
        let shapes = shapes::by_id();
        let mut buffer = RouteBuffer::new();
        let trip = trips::tram_12::oranienburger_tor_am_kupfergraben(time!(9:02:00));
        buffer.add_trip(
            stop_locations::tram_12::oranienburger_tor_am_kupfergraben(),
            shapes.bind(&"tram_12::oranienburger_tor_am_kupfergraben".into()),
            trip,
        );
        assert_eq!(buffer, route_buffers::tram_12::with_1_upstream(&shapes));
    }

    #[test]
    fn test_create_first_downstream_variant() {
        let shapes = shapes::by_id();
        let mut buffer = RouteBuffer::new();
        let trip = trips::tram_12::am_kupfergraben_oranienburger_tor(time!(8:34:00));
        buffer.add_trip(
            stop_locations::tram_12::am_kupfergraben_oranienburger_tor(),
            shapes.bind(&"tram_12::am_kupfergraben_oranienburger_tor".into()),
            trip,
        );
        assert_eq!(buffer, route_buffers::tram_12::with_1_downstream(&shapes));
    }

    #[test]
    fn test_append_to_upstream_variant() {
        let shapes = shapes::by_id();
        let mut buffer = route_buffers::tram_12::with_1_upstream(&shapes);
        let trip = trips::tram_12::oranienburger_tor_am_kupfergraben(time!(9:12:00));
        buffer.add_trip(
            stop_locations::tram_12::oranienburger_tor_am_kupfergraben(),
            shapes.bind(&"tram_12::oranienburger_tor_am_kupfergraben".into()),
            trip,
        );
        assert_eq!(buffer, route_buffers::tram_12::with_2_upstream(&shapes));
    }

    #[test]
    fn test_into_routes_same_terminus() {
        let shapes = shapes::by_id();
        let buffer = route_buffers::tram_12::with_1_upstream_1_downstream(&shapes);
        assert_eq_alternate!(
            buffer.into_routes(),
            vec![routes::tram_12::oranienburger_tor_am_kupfergraben()]
        );
    }

    #[test]
    fn test_into_routes_different_terminus() {
        let shapes = shapes::by_id();
        let buffer = RouteBuffer {
            upstream: vec![
                route_variants::tram_m10::clara_jaschke_str_warschauer_str(&shapes),
                route_variants::tram_m10::clara_jaschke_str_landsberger_allee_petersburger_str(
                    &shapes,
                ),
            ],
            downstream: vec![
                route_variants::tram_m10::landsberger_allee_petersburger_str_lueneburger_str(
                    &shapes,
                ),
                route_variants::tram_m10::warschauer_str_lueneburger_str(&shapes),
            ],
        };
        assert_eq_alternate!(
            buffer.into_routes(),
            vec![
                routes::tram_m10::clara_jaschke_str_landsberger_allee_petersburger_str(),
                routes::tram_m10::clara_jaschke_str_warschauer_str(),
            ]
        );
    }
}
