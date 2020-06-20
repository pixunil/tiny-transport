use chrono::Duration;
use itertools::Itertools;

use super::Node;
use simulation::Direction;

#[derive(Debug)]
struct DrivingSegment<'a> {
    duration: &'a mut u32,
    weight: f64,
}

impl<'a> DrivingSegment<'a> {
    fn add_duration(&mut self, delta: i32) {
        *self.duration = (*self.duration as i32 + delta).max(1) as u32;
    }
}

struct State<'a> {
    offset: i32,
    dispatch_time: &'a mut u32,
    driving_before: Option<DrivingSegment<'a>>,
    driving_after: Option<DrivingSegment<'a>>,
}

impl<'a> State<'a> {
    const MAXIMUM_OFFSET_VALUE: i32 = 25;

    fn new(dispatch_time: &'a mut u32) -> Self {
        Self {
            offset: 0,
            dispatch_time,
            driving_before: None,
            driving_after: None,
        }
    }

    fn step(&mut self, driving_segment: Option<DrivingSegment<'a>>) {
        self.driving_before = self.driving_after.take();
        self.driving_after = driving_segment;
    }

    fn subtract_stop_duration(&mut self, added_stop_duration: u32) {
        match (&mut self.driving_before, &mut self.driving_after) {
            (None, None) => unreachable!(),
            (None, Some(_)) => *self.dispatch_time -= added_stop_duration,
            (Some(driving_before), Some(driving_after)) => {
                let delta_min =
                    -Self::MAXIMUM_OFFSET_VALUE - self.offset - added_stop_duration as i32 / 2;
                let delta_max = delta_min + 2 * Self::MAXIMUM_OFFSET_VALUE;

                let factor_before = (*driving_after.duration as f64 - added_stop_duration as f64)
                    * driving_before.weight;
                let factor_after = *driving_before.duration as f64 * driving_after.weight;
                let total_weight = driving_before.weight + driving_after.weight;
                let delta = (((factor_before - factor_after) / total_weight) as i32)
                    .max(delta_min)
                    .min(delta_max);

                driving_before.add_duration(delta);
                driving_after.add_duration(-delta - added_stop_duration as i32);
                self.offset += delta + added_stop_duration as i32 / 2;
            }
            (Some(_), None) => {}
        }
    }
}

#[derive(Debug)]
pub(super) struct Scheduler {
    upstream_weights: Vec<f64>,
    downstream_weights: Vec<f64>,
}

impl Scheduler {
    const MINIMUM_STOP_DURATION: u32 = 20;

    pub(super) fn new(nodes: &[Node]) -> Self {
        Self {
            upstream_weights: Node::segment_weights(nodes, Direction::Upstream),
            downstream_weights: Node::segment_weights(nodes, Direction::Downstream),
        }
    }

    fn weights(&self, direction: Direction) -> impl Iterator<Item = f64> + '_ {
        match direction {
            Direction::Upstream => self.upstream_weights.iter().copied(),
            Direction::Downstream => self.downstream_weights.iter().copied(),
        }
    }

    pub(in crate::trip) fn process(
        &self,
        direction: Direction,
        durations: &[Duration],
    ) -> Vec<u32> {
        let mut durations = durations
            .iter()
            .map(|duration| duration.num_seconds() as u32)
            .collect::<Vec<_>>();

        let added_stop_durations = durations
            .iter_mut()
            .skip(1)
            .step_by(2)
            .map(|duration| {
                let missing_stop_duration = Self::MINIMUM_STOP_DURATION
                    .checked_sub(*duration)
                    .unwrap_or(0);
                *duration += missing_stop_duration;
                missing_stop_duration
            })
            .collect::<Vec<_>>();

        let mut iter = durations.iter_mut().step_by(2);
        let dispatch_time = iter.next().unwrap();
        let mut driving_segments = iter
            .zip_eq(self.weights(direction))
            .map(|(duration, weight)| DrivingSegment { duration, weight });

        let mut state = State::new(dispatch_time);
        for added_stop_duration in added_stop_durations {
            state.step(driving_segments.next());
            if added_stop_duration > 0 {
                state.subtract_stop_duration(added_stop_duration);
            }
        }

        durations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trip::fixtures::nodes;
    use simulation::Directions;

    macro_rules! times {
        (@time Duration, $time:expr) => { Duration::seconds($time) };
        (@time $type:ty, $time:expr) => { $time as $type };
        (@seconds $minute:literal, $second:literal) => { times!(@seconds 0, $minute, $second) };
        (@seconds $hour:literal, $minute:literal, $second:literal) => {
            $hour * 3600 + $minute * 60 + $second
        };
        ($type:ident; $( $( $(:)? $time:literal )* ),* ) => {
            vec![
                $( times!(@time $type, times!(@seconds $( $time ),* )) ),*
            ]
        };
    }

    #[test]
    fn test_sufficient_stop_times() {
        let durations = times!(Duration; 7:24:54, 0:30, 1:30, 0:48, 1:54, 0:36, 2:06, 0:30);
        let nodes = nodes::s3::hackescher_markt_bellevue(Directions::Both);
        let scheduler = Scheduler::new(&nodes);
        assert_eq!(
            scheduler.process(Direction::Upstream, &durations),
            times!(u32; 7:24:54, 0:30, 1:30, 0:48, 1:54, 0:36, 2:06, 0:30)
        )
    }

    #[test]
    fn test_no_stop_times_upstream() {
        let durations = times![Duration; 9:02:00, 0:00, 2:00, 0:00, 2:00, 0:00, 1:00, 0:00];
        let nodes = nodes::tram_12::oranienburger_tor_am_kupfergraben(Directions::Both);
        let scheduler = Scheduler::new(&nodes);
        assert_eq!(
            scheduler.process(Direction::Upstream, &durations),
            times!(u32; 9:01:40, 0:20, 2:15, 0:20, 1:05, 0:20, 1:00, 0:20)
        );
    }

    #[test]
    fn test_no_stop_times_downstream() {
        let durations = times![Duration; 8:34:00, 0:00, 1:00, 0:00, 3:00, 0:00, 2:00, 0:00];
        let nodes = nodes::tram_12::oranienburger_tor_am_kupfergraben(Directions::Both);
        let scheduler = Scheduler::new(&nodes);
        assert_eq!(
            scheduler.process(Direction::Downstream, &durations),
            times!(u32; 8:33:40, 0:20, 1:15, 0:20, 1:33, 0:20, 2:32, 0:20)
        );
    }
}
