use chrono::NaiveDate;

use super::{Node, Scheduler, Trip};
use crate::location::Linearizer;

#[derive(Debug, PartialEq)]
pub struct Route {
    nodes: Vec<Node>,
    trips: Vec<Trip>,
}

impl Route {
    pub(super) fn new(nodes: Vec<Node>, trips: Vec<Trip>) -> Route {
        Route { nodes, trips }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter()
    }

    pub(crate) fn num_trips_at(&self, date: NaiveDate) -> usize {
        self.trips
            .iter()
            .filter(|trip| trip.available_at(date))
            .count()
    }

    pub(crate) fn store_nodes(&self, linearizer: &mut Linearizer) -> Vec<storage::Node> {
        self.nodes
            .iter()
            .map(|node| node.store(linearizer))
            .collect()
    }

    pub(crate) fn store_trains(
        &self,
        date: NaiveDate,
        scheduler: &mut Scheduler,
    ) -> Vec<storage::Train> {
        scheduler.update_weights(&self.nodes);
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
            use test_utils::time;
            vec![ $( trips::$line::$route(time!($($time),*)) ),* ]
        }};
        ($( $line:ident : { $( $route:ident: $upstream:ident, $upstream_times:tt, $downstream:ident, $downstream_times:tt );* $(;)? } ),* $(,)?) => {
            $(
                pub(crate) mod $line {
                    use crate::fixtures::nodes;
                    use crate::trip::Route;
                    use simulation::Directions;

                    $(
                        pub(crate) fn $route() -> Route {
                            let mut trips = routes!(@trips $line, $upstream, $upstream_times);
                            trips.append(&mut routes!(@trips $line, $downstream, $downstream_times));
                            Route {
                                nodes: nodes::$line::$route(Directions::Both),
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
            clara_jaschke_str_warschauer_str:
                clara_jaschke_str_warschauer_str, [],
                warschauer_str_lueneburger_str, [];
            clara_jaschke_str_landsberger_allee_petersburger_str:
                clara_jaschke_str_landsberger_allee_petersburger_str, [],
                landsberger_allee_petersburger_str_lueneburger_str, [];
        },
        tram_12: {
            oranienburger_tor_am_kupfergraben:
                oranienburger_tor_am_kupfergraben, [9:02:00],
                am_kupfergraben_oranienburger_tor, [8:34:00];
        },
    }
}
