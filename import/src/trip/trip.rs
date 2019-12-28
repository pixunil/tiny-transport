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
