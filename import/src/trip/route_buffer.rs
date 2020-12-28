use std::rc::Rc;

use super::{Route, RouteVariant};
use crate::location::Location;
use crate::path::StopPlacer;
use crate::shape::{ShapeId, Shapes};
use simulation::Direction;

#[derive(Debug, PartialEq)]
pub(super) struct RouteBuffer {
    upstream: Vec<RouteVariant>,
    downstream: Vec<RouteVariant>,
}

impl RouteBuffer {
    pub(super) fn new() -> Self {
        Self {
            upstream: Vec::new(),
            downstream: Vec::new(),
        }
    }

    pub(super) fn retrieve_or_create_variant(
        &mut self,
        locations: Vec<Rc<Location>>,
        shapes: &Shapes,
        shape_id: ShapeId,
        direction: Direction,
    ) -> &mut RouteVariant {
        let variants = match direction {
            Direction::Upstream => &mut self.upstream,
            Direction::Downstream => &mut self.downstream,
        };

        let shape = &shapes[&shape_id];

        let pos = variants
            .iter()
            .position(|variant| variant.matches(&locations, shape));

        match pos {
            Some(pos) => &mut variants[pos],
            None => {
                let variant = RouteVariant::new(locations, shape.clone());
                variants.push(variant);
                variants.last_mut().unwrap()
            }
        }
    }

    pub(super) fn into_routes(self, placer: &mut StopPlacer) -> Vec<Route> {
        self.upstream
            .into_iter()
            .chain(self.downstream)
            .map(|variant| variant.single(placer))
            .collect()
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
        tram_m10: {
            clara_jaschke_str_warschauer_str => [clara_jaschke_str_warschauer_str], [],
            warschauer_str_lueneburger_str => [], [warschauer_str_lueneburger_str],
        },
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
    use crate::fixtures::{paths, route_buffers, route_variants, routes, shapes, stop_locations};
    use test_utils::assert_eq_alternate;

    #[test]
    fn test_create_first_upstream_variant() {
        let shapes = shapes::by_id();
        let mut buffer = RouteBuffer::new();
        buffer.retrieve_or_create_variant(
            stop_locations::tram_m10::clara_jaschke_str_warschauer_str(),
            &shapes,
            "tram_m10::clara_jaschke_str_warschauer_str".into(),
            Direction::Upstream,
        );
        assert_eq!(
            buffer,
            route_buffers::tram_m10::clara_jaschke_str_warschauer_str(&shapes)
        );
    }

    #[test]
    fn test_create_first_downstream_variant() {
        let shapes = shapes::by_id();
        let mut buffer = RouteBuffer::new();
        buffer.retrieve_or_create_variant(
            stop_locations::tram_m10::warschauer_str_lueneburger_str(),
            &shapes,
            "tram_m10::warschauer_str_lueneburger_str".into(),
            Direction::Downstream,
        );
        assert_eq!(
            buffer,
            route_buffers::tram_m10::warschauer_str_lueneburger_str(&shapes)
        );
    }

    #[test]
    fn test_append_to_upstream_variant() {
        let shapes = shapes::by_id();
        let mut buffer = route_buffers::tram_m10::clara_jaschke_str_warschauer_str(&shapes);
        buffer.retrieve_or_create_variant(
            stop_locations::tram_m10::clara_jaschke_str_warschauer_str(),
            &shapes,
            "tram_m10::clara_jaschke_str_warschauer_str".into(),
            Direction::Upstream,
        );
        assert_eq!(
            buffer,
            route_buffers::tram_m10::clara_jaschke_str_warschauer_str(&shapes)
        );
    }

    #[test]
    fn test_into_routes_same_terminus() {
        let shapes = shapes::by_id();
        let buffer = route_buffers::tram_12::with_1_upstream_1_downstream(&shapes);
        let mut placer = StopPlacer::new(&shapes.segments());
        let (_, segment_ids) = paths::tram_12::segments();
        assert_eq_alternate!(
            buffer.into_routes(&mut placer),
            vec![
                routes::tram_12::oranienburger_tor_am_kupfergraben(&segment_ids),
                routes::tram_12::am_kupfergraben_oranienburger_tor(&segment_ids),
            ]
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
                route_variants::tram_m10::warschauer_str_lueneburger_str(&shapes),
                route_variants::tram_m10::landsberger_allee_petersburger_str_lueneburger_str(
                    &shapes,
                ),
            ],
        };
        let mut placer = StopPlacer::new(shapes.segments());
        let (_, segment_ids) = paths::tram_m10::segments();
        assert_eq_alternate!(
            buffer.into_routes(&mut placer),
            vec![
                routes::tram_m10::clara_jaschke_str_warschauer_str(&segment_ids),
                routes::tram_m10::clara_jaschke_str_landsberger_allee_petersburger_str(
                    &segment_ids
                ),
                routes::tram_m10::warschauer_str_lueneburger_str(&segment_ids),
                routes::tram_m10::landsberger_allee_petersburger_str_lueneburger_str(&segment_ids),
            ]
        );
    }
}
