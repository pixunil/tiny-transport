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
mod tests {
    use super::*;

    #[macro_export]
    macro_rules! trip {
        ($direction:ident, $service:ident, [$($duration:expr),*]) => (
            Trip::new(
                Direction::$direction,
                Rc::new($crate::service!($service)),
                vec![$(chrono::Duration::minutes($duration)),*],
            )
        );
        (blue, Upstream, $start:expr) => (
            $crate::trip!(Upstream, mon_fri, [$start, 0, 4, 1, 4, 0])
        );
        (blue, Downstream, $start:expr) => (
            $crate::trip!(Downstream, mon_fri, [$start, 0, 4, 1, 4, 0])
        );
    }

    #[test]
    fn test_available_at() {
        let trip = trip!(blue, Upstream, 1);
        let date = NaiveDate::from_ymd(2019, 1, 7);
        assert!(trip.available_at(date));
    }
}
