use std::iter;

use itertools::Itertools;

use serde_derive::{Deserialize, Serialize};

use crate::schedule::Schedule;
use simulation::line::Kind;
use simulation::path::Node;
use simulation::Direction;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Train {
    direction: Direction,
    start_time: u32,
    schedule: usize,
}

impl Train {
    pub fn new(direction: Direction, start_time: u32, schedule: usize) -> Train {
        Train {
            direction,
            start_time,
            schedule,
        }
    }

    pub fn load(self, kind: Kind, nodes: &[&Node], schedules: &[Schedule]) -> simulation::Train {
        let durations = self.interpolate_times(nodes, schedules);
        simulation::Train::new(kind, self.direction, durations)
    }

    fn interpolate_times(&self, nodes: &[&Node], schedules: &[Schedule]) -> Vec<u32> {
        let stop_positions = nodes.iter().positions(|node| node.is_stop());

        let mut durations = vec![self.start_time];
        self.fill_before_dispatch(nodes, &mut durations);

        let schedule = &schedules[self.schedule];
        for ((stopping, driving), (start, end)) in
            schedule.into_iter().zip_eq(stop_positions.tuple_windows())
        {
            self.fill_driving(*stopping, *driving, &nodes[start..=end], &mut durations);
        }
        durations.push(schedule.stop_duration_at_terminus());

        self.fill_after_terminus(nodes, &mut durations);
        durations
    }

    fn fill_before_dispatch(&self, nodes: &[&Node], durations: &mut Vec<u32>) {
        let count = nodes.iter().position(|node| node.is_stop()).unwrap();
        durations.extend(iter::repeat(0).take(count));
    }

    fn fill_driving(&self, stopping: u32, driving: u32, nodes: &[&Node], durations: &mut Vec<u32>) {
        durations.push(stopping);

        let total_distance = self.segments_between(nodes).sum::<f32>();
        let times = self.segments_between(nodes).map(|distance| {
            let travelled = distance / total_distance;
            (driving as f32 * travelled).round() as u32
        });
        durations.extend(times);
    }

    fn segments_between<'a>(&'a self, nodes: &'a [&Node]) -> impl Iterator<Item = f32> + 'a {
        nodes
            .iter()
            .tuple_windows()
            .map(|(before, after)| na::distance(&before.position(), &after.position()))
    }

    fn fill_after_terminus(&self, nodes: &[&Node], durations: &mut Vec<u32>) {
        let count = nodes.iter().rev().position(|node| node.is_stop()).unwrap();
        durations.extend(iter::repeat(0).take(count));
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    macro_rules! trips {
        ($( $line:ident: { $( $trip:ident : $direction:ident ),* $(,)? } ),* $(,)?) => (
            $(
                pub mod $line {
                    use std::ops::Index;

                    use crate::train::*;
                    use simulation::Direction;

                    $(
                        pub fn $trip<'a>(
                            start_time: u32,
                            schedule_ids: &impl Index<&'a str, Output = usize>,
                        ) -> Train {
                            Train {
                                direction: Direction::$direction,
                                start_time,
                                schedule: schedule_ids[stringify!($trip)],
                            }
                        }
                    )*
                }
            )*
        );
    }

    trips! {
        s3: {
            hackescher_markt_bellevue: Upstream,
            bellevue_hackescher_markt: Downstream,
        },
        u6: {
            naturkundemuseum_franzoesische_str: Upstream,
            franzoesische_str_naturkundemuseum: Downstream,
        },
        tram_m5: {
            zingster_str_prerower_platz: Upstream,
        },
        tram_12: {
            oranienburger_tor_am_kupfergraben: Upstream,
            am_kupfergraben_oranienburger_tor: Downstream,
        },
        bus_m82: {
            weskammstr_waldsassener_str: Upstream,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::trains;
    use common::{fixtures_with_ids, time, times};

    macro_rules! nodes {
        ($line:ident :: $route:ident) => {{
            let (segments, segment_ids) = simulation::fixtures::paths::$line::segments();
            let path = simulation::fixtures::paths::$line::$route(&segment_ids);
            (path, segments)
        }};
    }

    #[test]
    fn test_time_interpolation_upstream() {
        let (schedules, schedule_ids) = fixtures_with_ids!(schedules::{
            oranienburger_tor_am_kupfergraben,
        });
        let (path, segments) = nodes!(tram_12::oranienburger_tor_am_kupfergraben);
        let train =
            trains::tram_12::oranienburger_tor_am_kupfergraben(time!(9:01:40), &schedule_ids);
        assert_eq!(
            train.interpolate_times(&path.nodes(&segments), &schedules),
            times![9:01:40, 0:20, 0:27, 1:21, 0:27, 0:20, 0:25, 0:13, 0:13, 0:13,
                0:20, 1:00, 0:20]
        );
    }

    #[test]
    fn test_time_interpolation_downstream() {
        let (schedules, schedule_ids) = fixtures_with_ids!(schedules::{
            am_kupfergraben_oranienburger_tor,
        });
        let (path, segments) = nodes!(tram_12::am_kupfergraben_oranienburger_tor);
        let train =
            trains::tram_12::am_kupfergraben_oranienburger_tor(time!(8:33:40), &schedule_ids);
        assert_eq!(
            train.interpolate_times(&path.nodes(&segments), &schedules),
            times![8:33:40, 0:20, 0:19, 0:22, 0:12, 0:22, 0:20, 0:25, 0:12, 0:20,
                0:12, 0:24, 0:20, 0:30, 1:31, 0:30, 0:20]
        );
    }

    #[test]
    fn test_clamp_before_dispatch() {
        let (schedules, schedule_ids) = fixtures_with_ids!(schedules::{
            zingster_str_prerower_platz,
        });
        let (path, segments) = nodes!(tram_m5::zingster_str_prerower_platz);
        let train = trains::tram_m5::zingster_str_prerower_platz(time!(8:13:00), &schedule_ids);
        assert_eq!(
            train.interpolate_times(&path.nodes(&segments), &schedules),
            times!(8:13:00, 0:00, 0:00, 1:00, 0:00, 1:00, 0:00, 0:48, 1:12, 0:00)
        );
    }

    #[test]
    fn test_clamp_after_terminus() {
        let (schedules, schedule_ids) = fixtures_with_ids!(schedules::{
            weskammstr_waldsassener_str,
        });
        let (path, segments) = nodes!(bus_m82::weskammstr_waldsassener_str);
        let train = trains::bus_m82::weskammstr_waldsassener_str(time!(9:46:00), &schedule_ids);
        assert_eq!(
            train.interpolate_times(&path.nodes(&segments), &schedules),
            times![9:46:00, 0:00, 0:00, 0:15, 0:15, 0:00, 0:14, 0:14, 0:17, 0:14,
                0:00, 0:00, 0:00]
        );
    }

    #[test]
    fn test_load() {
        let (schedules, schedule_ids) = fixtures_with_ids!(schedules::{
            oranienburger_tor_am_kupfergraben,
        });
        let (path, segments) = nodes!(tram_12::oranienburger_tor_am_kupfergraben);
        let train =
            trains::tram_12::oranienburger_tor_am_kupfergraben(time!(8:13:00), &schedule_ids);
        assert_eq!(
            train.load(Kind::Tram, &path.nodes(&segments), &schedules),
            simulation::fixtures::trains::tram_12::oranienburger_tor_am_kupfergraben(
                time!(8:13:00)
            )
        );
    }
}
