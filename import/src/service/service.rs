use std::collections::HashSet;

use chrono::{NaiveDate, Datelike};

use crate::create_id_type;

create_id_type!(ServiceId);

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Service {
    start: NaiveDate,
    end: NaiveDate,
    weekdays: [bool; 7],
    added: HashSet<NaiveDate>,
    removed: HashSet<NaiveDate>,
}

impl Service {
    pub(crate) fn new(start: NaiveDate, end: NaiveDate, weekdays: [bool; 7]) -> Service {
        Service {
            start,
            end,
            weekdays,
            added: HashSet::new(),
            removed: HashSet::new(),
        }
    }

    pub(super) fn add_date(&mut self, date: NaiveDate) {
        self.added.insert(date);
    }

    pub(super) fn remove_date(&mut self, date: NaiveDate) {
        self.removed.insert(date);
    }

    pub(crate) fn available_at(&self, date: NaiveDate) -> bool {
        self.added.contains(&date) || (!self.removed.contains(&date) && self.regularly_available_at(date))
    }

    fn regularly_available_at(&self, date: NaiveDate) -> bool {
        let day = date.weekday().num_days_from_monday() as usize;
        self.start <= date && date <= self.end && self.weekdays[day]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::service;

    #[test]
    fn test_regularly_available() {
        let service = service!(mon-fri);
        let date = NaiveDate::from_ymd(2019, 1, 7);
        assert!(service.regularly_available_at(date));
        assert!(service.available_at(date));
    }

    #[test]
    fn test_regularly_unavailable() {
        let service = service!(mon-fri);
        let date = NaiveDate::from_ymd(2019, 1, 5);
        assert!(!service.regularly_available_at(date));
        assert!(!service.available_at(date));
    }

    #[test]
    fn test_exceptionally_available() {
        let mut service = service!(mon-fri);
        let date = NaiveDate::from_ymd(2019, 1, 5);
        service.add_date(date);
        assert!(!service.regularly_available_at(date));
        assert!(service.available_at(date));
    }

    #[test]
    fn test_exceptionally_unavailable() {
        let mut service = service!(mon-fri);
        let date = NaiveDate::from_ymd(2019, 1, 7);
        service.remove_date(date);
        assert!(service.regularly_available_at(date));
        assert!(!service.available_at(date));
    }
}
