use std::rc::Rc;

use chrono::{Duration, NaiveDate};

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

    pub(super) fn store(&self) -> storage::Train {
        let durations = self
            .durations
            .iter()
            .map(|duration| duration.num_seconds() as u32)
            .collect();
        storage::Train::new(self.direction, durations)
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
                        pub(in crate::trip) fn $trip(hour: i64, minute: f64) -> Trip {
                            let start = hour * 3600 + (minute * 60.0) as i64;
                            Trip {
                                direction: Direction::$direction,
                                service: Rc::new(services::$service()),
                                durations: vec![
                                    Duration::seconds(start),
                                    $( Duration::seconds(($time as f64 * 60.0) as i64) ),*
                                ],
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
        let trip = trips::tram_12::oranienburger_tor_am_kupfergraben(9, 2.0);
        let date = NaiveDate::from_ymd(2019, 1, 7);
        assert!(trip.available_at(date));
    }

    #[test]
    fn test_store() {
        assert_eq!(
            trips::tram_12::oranienburger_tor_am_kupfergraben(9, 2.0).store(),
            storage::fixtures::trains::tram_12::oranienburger_tor_am_kupfergraben(9, 2.0)
        );
    }
}
