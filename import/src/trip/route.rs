use chrono::NaiveDate;

use super::{Node, Trip};
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

    pub(crate) fn store_trains(&self, date: NaiveDate) -> Vec<storage::Train> {
        self.trips
            .iter()
            .filter(|trip| trip.available_at(date))
            .map(Trip::store)
            .collect()
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    macro_rules! routes {
        (@trips $line:ident, $route:ident, [$( $( $(:)? $time:literal )* ),* $(,)?]) => {
            $( trips::$line::$route(time!($($time),*)) ),*
        };
        ($( $line:ident : { $( $route:ident: $upstream:ident, $upstream_times:tt, $downstream:ident, $downstream_times:tt );* $(;)? } ),* $(,)?) => {
            $(
                pub(crate) mod $line {
                    use crate::trip::fixtures::{nodes, trips};
                    use crate::trip::Route;
                    use simulation::Directions;
                    use test_utils::time;

                    $(
                        pub(crate) fn $route() -> Route {
                            Route {
                                nodes: nodes::$line(Directions::Both),
                                trips: vec![
                                    routes!(@trips $line, $upstream, $upstream_times),
                                    routes!(@trips $line, $downstream, $downstream_times),
                                ],
                            }
                        }
                    )*
                }
            )*
        };
    }

    routes! {
        tram_12: {
            oranienburger_tor_am_kupfergraben:
                oranienburger_tor_am_kupfergraben, [9:02:00],
                am_kupfergraben_oranienburger_tor, [8:34:00];
        },
    }
}
