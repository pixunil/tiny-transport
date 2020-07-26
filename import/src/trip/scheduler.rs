use std::collections::HashMap;

use chrono::Duration;
use itertools::Itertools;

use super::{Node, Schedule};
use simulation::Direction;

#[derive(Debug)]
pub(crate) struct Scheduler {
    upstream_weights: Vec<f64>,
    downstream_weights: Vec<f64>,
    schedules: HashMap<Schedule, usize>,
}

impl Scheduler {
    pub(crate) fn new() -> Self {
        Self {
            upstream_weights: Vec::new(),
            downstream_weights: Vec::new(),
            schedules: HashMap::new(),
        }
    }

    pub(super) fn update_weights(&mut self, nodes: &[Node]) {
        self.upstream_weights = Node::segment_weights(nodes, Direction::Upstream);
        self.downstream_weights = Node::segment_weights(nodes, Direction::Downstream);
    }

    fn weights(&self, direction: Direction) -> impl Iterator<Item = f64> + '_ {
        match direction {
            Direction::Upstream => self.upstream_weights.iter().copied(),
            Direction::Downstream => self.downstream_weights.iter().copied(),
        }
    }

    pub(super) fn process(&mut self, direction: Direction, durations: &[Duration]) -> (u32, usize) {
        let mut durations = durations
            .iter()
            .map(|duration| duration.num_seconds() as u32);
        let start_time = durations.next().unwrap();
        let mut schedule = Schedule::new(durations);
        let start_time_offset = schedule.adjust_stop_durations(self.weights(direction));
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
    use crate::fixtures::nodes;
    use simulation::Directions;
    use test_utils::{time, times};

    #[test]
    fn test_single_schedule() {
        let mut scheduler = Scheduler::new();
        let nodes = nodes::s3::hackescher_markt_bellevue(Directions::Both);
        scheduler.update_weights(&nodes);
        let durations = times!(Duration; 7:24:54, 0:30, 1:30, 0:48, 1:54, 0:36, 2:06, 0:30);
        let (start_time, schedule_id) = scheduler.process(Direction::Upstream, &durations);
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
        let nodes = nodes::tram_12::oranienburger_tor_am_kupfergraben(Directions::Both);
        scheduler.update_weights(&nodes);
        let mut durations = times!(Duration; 9:02:00, 0:00, 2:00, 0:00, 2:00, 0:00, 1:00, 0:00);
        let (start_time_a, schedule_id_a) = scheduler.process(Direction::Upstream, &durations);
        durations[0] = time!(Duration; 9:12:00);
        let (start_time_b, schedule_id_b) = scheduler.process(Direction::Upstream, &durations);
        assert_eq!(start_time_a, time!(9:01:40));
        assert_eq!(start_time_b, time!(9:11:40));
        assert_eq!(schedule_id_a, schedule_id_b);
        assert_eq!(
            scheduler.schedules(),
            vec![storage::fixtures::schedules::oranienburger_tor_am_kupfergraben()]
        );
    }
}
