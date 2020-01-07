use std::rc::Rc;

use chrono::{Duration, NaiveDate};

use simulation::Direction;
use crate::service::Service;

#[derive(Debug, PartialEq)]
pub(super) struct Trip {
    direction: Direction,
    service: Rc<Service>,
    durations: Vec<Duration>,
}

impl Trip {
    pub(super) fn new(direction: Direction, service: Rc<Service>, durations: Vec<Duration>) -> Self {
        Self {
            direction,
            service,
            durations,
        }
    }

    pub(super) fn direction(&self) -> Direction {
        self.direction
    }

    pub(super) fn freeze(&self) -> serialization::Train {
        let durations = self.durations.iter()
            .map(|duration| duration.num_seconds() as u32)
            .collect();
        serialization::Train::new(self.direction, durations)
    }

    pub(super) fn available_at(&self, date: NaiveDate) -> bool {
        self.service.available_at(date)
    }
}

#[cfg(test)]
pub(super) mod fixtures {
    macro_rules! trips {
        ($($line:ident: {$($trip:ident => $direction:ident, $service:ident, [$($time:expr),*]);* $(;)?}),* $(,)?) => (
            $(
                pub(in crate::trip) mod $line {
                    use simulation::Direction;
                    use crate::trip::fixtures::*;
                    use crate::trip::trip::*;

                    $(
                        pub(in crate::trip) fn $trip(start: i64) -> Trip {
                            Trip {
                                direction: Direction::$direction,
                                service: Rc::new(services::$service()),
                                durations: vec![Duration::minutes(start), $(Duration::minutes($time)),*],
                            }
                        }
                    )*
                }
            )*
        );
    }

    trips! {
        tram_12: {
            oranienburger_tor_am_kupfergraben => Upstream, mon_fri, [0, 2, 0, 2, 0, 1, 0];
            am_kupfergraben_oranienburger_tor => Downstream, mon_fri, [0, 1, 0, 3, 0, 2, 0];
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::trip::fixtures::*;

    #[test]
    fn test_available_at() {
        let trip = trips::tram_12::oranienburger_tor_am_kupfergraben(542);
        let date = NaiveDate::from_ymd(2019, 1, 7);
        assert!(trip.available_at(date));
    }
}
