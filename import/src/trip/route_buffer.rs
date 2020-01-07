use std::rc::Rc;

use itertools::{Itertools, EitherOrBoth::*};

use simulation::Direction;
use crate::shape::Shape;
use crate::location::Location;
use super::{RouteVariant, Route, Trip};

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

    pub(super) fn add_trip(&mut self, locations: Vec<Rc<Location>>, shape: &Shape, trip: Trip) {
        let variants = match trip.direction() {
            Direction::Upstream => &mut self.upstream,
            Direction::Downstream => &mut self.downstream,
        };
        let variant = variants.iter_mut()
            .find(|variant| variant.matches(&locations, shape));

        match variant {
            Some(variant) => {
                variant.add_trip(trip);
            },
            None => {
                let mut variant = RouteVariant::new(locations, shape.clone());
                variant.add_trip(trip);
                variants.push(variant);
            }
        }
    }

    pub(super) fn into_routes(mut self) -> impl Iterator<Item = Route> {
        self.upstream.sort_by_key(|variant| variant.trips.len());
        self.downstream.sort_by_key(|variant| variant.trips.len());

        self.upstream.into_iter().zip_longest(self.downstream)
            .map(|variants| {
                match variants {
                    Both(upstream, downstream) => upstream.merge(downstream),
                    Left(upstream) => upstream.single(Direction::Upstream),
                    Right(downstream) => downstream.single(Direction::Downstream),
                }
            })
    }
}

#[cfg(test)]
pub(super) mod fixtures {
    macro_rules! route_buffers {
        ($($line:ident: {$($route:ident => [$($upstream:ident),* $(,)?], [$($downstream:ident),* $(,)?]),* $(,)?}),* $(,)?) => (
            $(
                pub(in crate::trip) mod $line {
                    use crate::trip::fixtures::*;
                    use crate::trip::route_buffer::*;

                    $(
                        pub(in crate::trip) fn $route() -> RouteBuffer {
                            RouteBuffer {
                                upstream: vec![$(route_variants::$line::$upstream()),*],
                                downstream: vec![$(route_variants::$line::$downstream()),*],
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

    use crate::trip::fixtures::*;

    #[test]
    fn test_create_first_upstream_variant() {
        let mut buffer = RouteBuffer::new();
        let trip = trips::tram_12::oranienburger_tor_am_kupfergraben(542);
        buffer.add_trip(stop_locations::tram_12::oranienburger_tor_am_kupfergraben(), &shapes::tram_12::oranienburger_tor_am_kupfergraben(), trip);
        assert_eq!(buffer, route_buffers::tram_12::with_1_upstream());
    }

    #[test]
    fn test_create_first_downstream_variant() {
        let mut buffer = RouteBuffer::new();
        let trip = trips::tram_12::am_kupfergraben_oranienburger_tor(514);
        buffer.add_trip(stop_locations::tram_12::am_kupfergraben_oranienburger_tor(), &shapes::tram_12::am_kupfergraben_oranienburger_tor(), trip);
        assert_eq!(buffer, route_buffers::tram_12::with_1_downstream());
    }

    #[test]
    fn test_append_to_upstream_variant() {
        let mut buffer = route_buffers::tram_12::with_1_upstream();
        let trip = trips::tram_12::oranienburger_tor_am_kupfergraben(552);
        buffer.add_trip(stop_locations::tram_12::oranienburger_tor_am_kupfergraben(), &shapes::tram_12::oranienburger_tor_am_kupfergraben(), trip);
        assert_eq!(buffer, route_buffers::tram_12::with_2_upstream());
    }
}
