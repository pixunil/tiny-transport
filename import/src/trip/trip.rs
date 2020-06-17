use std::rc::Rc;

use chrono::{Duration, NaiveDate};

use super::Scheduler;
use crate::service::Service;
use simulation::Direction;

#[derive(Debug, PartialEq)]
pub(super) struct Trip {
    direction: Direction,
    service: Rc<Service>,
    durations: Vec<Duration>,
}

impl Trip {
    pub(super) fn new(
        direction: Direction,
        service: Rc<Service>,
        durations: Vec<Duration>,
    ) -> Self {
        Self {
            direction,
            service,
            durations,
        }
    }

    pub(super) fn direction(&self) -> Direction {
        self.direction
    }

    pub(super) fn store(&self, scheduler: &Scheduler) -> storage::Train {
        let durations = scheduler.process(self.direction, &self.durations);
        storage::Train::new(self.direction, durations)
    }

    pub(super) fn available_at(&self, date: NaiveDate) -> bool {
        self.service.available_at(date)
    }
}

#[cfg(test)]
pub(super) mod fixtures {
    macro_rules! trips {
        ($(
            $line:ident: {
                $( $trip:ident => $direction:ident, $service:ident, $times:tt );* $(;)?
            }
        ),* $(,)?) => (
            $(
                pub(in crate::trip) mod $line {
                    use simulation::Direction;
                    use crate::trip::fixtures::*;
                    use crate::trip::trip::*;
                    use test_utils::times;

                    $(
                        pub(in crate::trip) fn $trip(start: i64) -> Trip {
                            Trip {
                                direction: Direction::$direction,
                                service: Rc::new(services::$service()),
                                durations: times!(Duration; start, $times),
                            }
                        }
                    )*
                }
            )*
        );
    }

    trips! {
        tram_12: {
            oranienburger_tor_am_kupfergraben => Upstream, mon_fri,
            [0:00, 2:00, 0:00, 2:00, 0:00, 1:00, 0:00];
            am_kupfergraben_oranienburger_tor => Downstream, mon_fri,
            [0:00, 1:00, 0:00, 3:00, 0:00, 2:00, 0:00];
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trip::fixtures::{nodes, trips};
    use simulation::Directions;
    use test_utils::time;

    #[test]
    fn test_available_at() {
        let trip = trips::tram_12::oranienburger_tor_am_kupfergraben(time!(9:02:00));
        let date = NaiveDate::from_ymd(2019, 1, 7);
        assert!(trip.available_at(date));
    }

    #[test]
    fn test_store() {
        let scheduler = Scheduler::new(&nodes::tram_12(Directions::UpstreamOnly));
        assert_eq!(
            trips::tram_12::oranienburger_tor_am_kupfergraben(time!(9:02:00)).store(&scheduler),
            storage::fixtures::trains::tram_12::oranienburger_tor_am_kupfergraben(time!(9:01:40))
        );
    }
}
