use chrono::NaiveDate;

use super::{Scheduler, Trip};
use crate::path::{Segment, SegmentedPath};

#[derive(Debug, PartialEq)]
pub struct Route {
    path: SegmentedPath,
    trips: Vec<Trip>,
}

impl Route {
    pub(super) fn new(path: SegmentedPath, trips: Vec<Trip>) -> Route {
        Route { path, trips }
    }

    pub fn path(&self) -> &SegmentedPath {
        &self.path
    }

    pub(crate) fn num_trips_at(&self, date: NaiveDate) -> usize {
        self.trips
            .iter()
            .filter(|trip| trip.available_at(date))
            .count()
    }

    pub(crate) fn store_trains(
        &self,
        date: NaiveDate,
        segments: &[Segment],
        scheduler: &mut Scheduler,
    ) -> Vec<storage::Train> {
        scheduler.update_weights(&self.path, segments);
        self.trips
            .iter()
            .filter(|trip| trip.available_at(date))
            .map(|trip| trip.store(scheduler))
            .collect()
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    macro_rules! routes {
        (@trips $line:ident, $route:ident, []) => { vec![] };
        (@trips $line:ident, $route:ident, [$( $( $(:)? $time:literal )* ),* $(,)?]) => {{
            use crate::fixtures::trips;
            use common::time;
            vec![ $( trips::$line::$route(time!($($time),*)) ),* ]
        }};
        ($( $line:ident : { $( $route:ident: $times:tt );* $(;)? } ),* $(,)?) => {
            $(
                pub(crate) mod $line {
                    use std::ops::Index;

                    use crate::fixtures::paths;
                    use crate::trip::Route;

                    $(
                        pub(crate) fn $route<'a>(
                            segments: &impl Index<&'a str, Output = usize>,
                        ) -> Route {
                            let trips = routes!(@trips $line, $route, $times);
                            Route {
                                path: paths::$line::$route(segments),
                                trips,
                            }
                        }
                    )*
                }
            )*
        };
    }

    routes! {
        tram_m10: {
            clara_jaschke_str_warschauer_str: [];
            warschauer_str_lueneburger_str: [];
            clara_jaschke_str_landsberger_allee_petersburger_str: [];
            landsberger_allee_petersburger_str_lueneburger_str: [];
        },
        tram_12: {
            oranienburger_tor_am_kupfergraben: [9:02:00];
            am_kupfergraben_oranienburger_tor: [8:34:00];
        },
    }
}
