use std::iter;
use std::rc::Rc;

use super::{Route, Trip};
use crate::location::Location;
use crate::path::StopPlacer;
use crate::shape::SegmentedShape;

#[derive(Debug, PartialEq)]
pub(super) struct RouteVariant {
    locations: Vec<Rc<Location>>,
    shape: SegmentedShape,
    trips: Vec<Trip>,
}

impl RouteVariant {
    pub(super) fn new(locations: Vec<Rc<Location>>, shape: SegmentedShape) -> Self {
        Self {
            locations,
            shape,
            trips: Vec::new(),
        }
    }

    pub(super) fn matches(&self, locations: &[Rc<Location>], shape: &SegmentedShape) -> bool {
        self.locations == locations && &self.shape == shape
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

    pub(super) fn single(self, placer: &mut StopPlacer) -> Route {
        Route::new(placer.place_stops(&self.shape, &self.locations), self.trips)
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    macro_rules! route_variants {
        (@trips $line:ident, $route:ident, []) => { vec![] };
        (@trips $line:ident, $route:ident, [$( $( $(:)? $time:literal )* ),* $(,)?]) => {{
            use crate::fixtures::trips;
            use common::time;
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
                    use crate::fixtures::stop_locations;
                    use crate::shape::Shapes;
                    use crate::trip::route_variant::*;
                    use common::join;

                    $(
                        pub(in crate::trip) fn $name(shapes: &Shapes) -> RouteVariant {
                            RouteVariant {
                                locations: stop_locations::$line::$route(),
                                shape: shapes[&join!($line, $route).into()].clone(),
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
