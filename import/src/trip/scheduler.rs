use std::collections::HashMap;

use chrono::Duration;
use itertools::Itertools;

use super::Schedule;
use crate::path::{Segment, SegmentedPath};

#[derive(Debug)]
pub(crate) struct Scheduler {
    weights: Vec<f64>,
    schedules: HashMap<Schedule, usize>,
}

impl Scheduler {
    pub(crate) fn new() -> Self {
        Self {
            weights: Vec::new(),
            schedules: HashMap::new(),
        }
    }

    pub(super) fn update_weights(&mut self, path: &SegmentedPath, segments: &[Segment]) {
        self.weights = path.segment_weights(segments);
    }

    pub(super) fn process(&mut self, durations: &[Duration]) -> (u32, usize) {
        let mut durations = durations
            .iter()
            .map(|duration| duration.num_seconds() as u32);
        let start_time = durations.next().unwrap();
        let mut schedule = Schedule::new(durations);
        let start_time_offset = schedule.adjust_stop_durations(self.weights.iter().copied());
        let len = self.schedules.len();
        let schedule_id = *self.schedules.entry(schedule).or_insert(len);
        ((start_time as i32 + start_time_offset) as u32, schedule_id)
    }

    pub(crate) fn schedules(self) -> Vec<storage::Schedule> {
        self.schedules
            .into_iter()
            .sorted_by_key(|(_, schedule_id)| *schedule_id)
            .map(|(schedule, _)| schedule.store())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::paths;
    use common::{time, times};

    #[test]
    fn test_single_schedule() {
        let mut scheduler = Scheduler::new();
        let (segments, segment_ids) = paths::s3::segments();
        let path = paths::s3::hackescher_markt_bellevue(&segment_ids);
        scheduler.update_weights(&path, &segments);
        let durations = times!(Duration; 7:24:54, 0:30, 1:30, 0:48, 1:54, 0:36, 2:06, 0:30);
        let (start_time, schedule_id) = scheduler.process(&durations);
        assert_eq!(start_time, time!(7:24:54));
        assert_eq!(schedule_id, 0);
        assert_eq!(
            scheduler.schedules(),
            vec![storage::fixtures::schedules::hackescher_markt_bellevue()]
        );
    }

    #[test]
    fn test_reuse_schedule() {
        let mut scheduler = Scheduler::new();
        let (segments, segment_ids) = paths::tram_12::segments();
        let path = paths::tram_12::oranienburger_tor_am_kupfergraben(&segment_ids);
        scheduler.update_weights(&path, &segments);
        let mut durations = times!(Duration; 9:02:00, 0:00, 2:00, 0:00, 2:00, 0:00, 1:00, 0:00);
        let (start_time_a, schedule_id_a) = scheduler.process(&durations);
        durations[0] = time!(Duration; 9:12:00);
        let (start_time_b, schedule_id_b) = scheduler.process(&durations);
        assert_eq!(start_time_a, time!(9:01:40));
        assert_eq!(start_time_b, time!(9:11:40));
        assert_eq!(schedule_id_a, schedule_id_b);
        assert_eq!(
            scheduler.schedules(),
            vec![storage::fixtures::schedules::oranienburger_tor_am_kupfergraben()]
        );
    }
}
