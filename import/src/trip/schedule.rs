use std::cmp::max;
use std::fmt;

use itertools::Itertools;

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
    start_time_offset: i32,
    driving_before: Option<DrivingSegment<'a>>,
    driving_after: Option<DrivingSegment<'a>>,
}

impl<'a> State<'a> {
    const MAXIMUM_OFFSET_VALUE: i32 = 25;

    fn new() -> Self {
        Self {
            offset: 0,
            start_time_offset: 0,
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
            (None, Some(_)) => self.start_time_offset = -(added_stop_duration as i32),
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

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) struct Schedule {
    driving_durations: Vec<(u32, u32)>,
    stop_duration_at_terminus: u32,
}

impl Schedule {
    const MINIMUM_STOP_DURATION: u32 = 20;

    pub(super) fn new(durations: impl Iterator<Item = u32> + fmt::Debug) -> Self {
        let mut durations = durations.tuples();
        let mut driving_durations = Vec::new();
        while let Some((stopping, driving)) = durations.next() {
            driving_durations.push((stopping, driving));
        }
        let stop_duration_at_terminus = durations.into_buffer().exactly_one().unwrap();
        Self {
            driving_durations,
            stop_duration_at_terminus,
        }
    }

    pub(super) fn adjust_stop_durations<'a>(
        &mut self,
        weights: impl Iterator<Item = f64> + 'a,
    ) -> i32 {
        let added_stop_durations = self
            .driving_durations
            .iter_mut()
            .map(|(stopping, _)| {
                let missing_stop_duration = Self::MINIMUM_STOP_DURATION.saturating_sub(*stopping);
                *stopping += missing_stop_duration;
                missing_stop_duration
            })
            .collect::<Vec<_>>();

        self.stop_duration_at_terminus =
            max(self.stop_duration_at_terminus, Self::MINIMUM_STOP_DURATION);

        let mut driving_segments = self
            .driving_durations
            .iter_mut()
            .zip_eq(weights)
            .map(|((_, duration), weight)| DrivingSegment { duration, weight });

        let mut state = State::new();
        for added_stop_duration in added_stop_durations {
            state.step(driving_segments.next());
            if added_stop_duration > 0 {
                state.subtract_stop_duration(added_stop_duration);
            }
        }
        state.start_time_offset
    }

    pub(crate) fn store(&self) -> storage::Schedule {
        storage::Schedule::new(
            self.driving_durations.clone(),
            self.stop_duration_at_terminus,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::paths;
    use common::{time, times};

    #[test]
    fn test_sufficient_stop_times() {
        let durations = times![0:30, 1:30, 0:48, 1:54, 0:36, 2:06, 0:30];
        let mut schedule = Schedule::new(durations.into_iter());
        let (segments, segment_ids) = paths::s3::segments();
        let weights = paths::s3::hackescher_markt_bellevue(&segment_ids).segment_weights(&segments);
        let start_time_offset = schedule.adjust_stop_durations(weights.into_iter());
        assert_eq!(start_time_offset, 0);
        let expected_durations = times![0:30, 1:30, 0:48, 1:54, 0:36, 2:06, 0:30];
        assert_eq!(schedule, Schedule::new(expected_durations.into_iter()));
    }

    #[test]
    fn test_no_stop_times_upstream() {
        let durations = times![0:00, 2:00, 0:00, 2:00, 0:00, 1:00, 0:00];
        let mut schedule = Schedule::new(durations.into_iter());
        let (segments, segment_ids) = paths::tram_12::segments();
        let weights = paths::tram_12::oranienburger_tor_am_kupfergraben(&segment_ids)
            .segment_weights(&segments);
        let start_time_offset = schedule.adjust_stop_durations(weights.into_iter());
        assert_eq!(start_time_offset, -20);
        let expected_durations = times![0:20, 2:15, 0:20, 1:05, 0:20, 1:00, 0:20];
        assert_eq!(schedule, Schedule::new(expected_durations.into_iter()));
    }

    #[test]
    fn test_no_stop_times_downstream() {
        let durations = times![0:00, 1:00, 0:00, 3:00, 0:00, 2:00, 0:00];
        let mut schedule = Schedule::new(durations.into_iter());
        let (segments, segment_ids) = paths::tram_12::segments();
        let weights = paths::tram_12::am_kupfergraben_oranienburger_tor(&segment_ids)
            .segment_weights(&segments);
        let start_time_offset = schedule.adjust_stop_durations(weights.into_iter());
        assert_eq!(start_time_offset, -20);
        let expected_durations = times![0:20, 1:15, 0:20, 1:33, 0:20, 2:32, 0:20];
        assert_eq!(schedule, Schedule::new(expected_durations.into_iter()));
    }
}
