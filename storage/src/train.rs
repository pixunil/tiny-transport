use std::iter;

use itertools::Itertools;

use serde_derive::{Deserialize, Serialize};

use simulation::line::Kind;
use simulation::{Direction, Node};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Train {
    direction: Direction,
    durations: Vec<u32>,
}

impl Train {
    pub fn new(direction: Direction, durations: Vec<u32>) -> Train {
        Train {
            direction,
            durations,
        }
    }

    pub fn load(self, kind: Kind, nodes: &[Node]) -> simulation::Train {
        let durations = self.interpolate_times(nodes.to_vec());
        simulation::Train::new(kind, self.direction, durations)
    }

    fn interpolate_times(&self, mut nodes: Vec<Node>) -> Vec<u32> {
        if self.direction == Direction::Downstream {
            nodes.reverse();
        }

        let stop_positions = nodes.iter().positions(|node| self.is_node_allowed(node));

        let mut durations = vec![self.durations[0]];
        self.fill_before_dispatch(&nodes, &mut durations);

        for (i, (start, end)) in stop_positions.tuple_windows().enumerate() {
            self.fill_driving(2 * i + 1, start, end, &nodes, &mut durations);
        }
        durations.push(*self.durations.last().unwrap());

        self.fill_after_terminus(&nodes, &mut durations);
        durations
    }

    fn is_node_allowed(&self, node: &Node) -> bool {
        node.is_stop() && node.allows(self.direction)
    }

    fn fill_before_dispatch(&self, nodes: &[Node], durations: &mut Vec<u32>) {
        let count = nodes
            .iter()
            .position(|node| self.is_node_allowed(node))
            .unwrap();
        durations.extend(iter::repeat(0).take(count));
    }

    fn fill_driving(
        &self,
        stop: usize,
        start: usize,
        end: usize,
        nodes: &[Node],
        durations: &mut Vec<u32>,
    ) {
        durations.push(self.durations[stop]);

        let duration = self.durations[stop + 1] as f32;

        let total_distance = self.segments_between(start, end, nodes).sum::<f32>();
        let times = self.segments_between(start, end, nodes).map(|distance| {
            let travelled = distance / total_distance;
            (duration * travelled).round() as u32
        });
        durations.extend(times);
    }

    fn segments_between<'a>(
        &'a self,
        start: usize,
        end: usize,
        nodes: &'a [Node],
    ) -> impl Iterator<Item = f32> + 'a {
        nodes[start..=end]
            .iter()
            .filter(move |node| node.allows(self.direction))
            .tuple_windows()
            .map(|(before, after)| na::distance(&before.position(), &after.position()))
    }

    fn fill_after_terminus(&self, nodes: &[Node], durations: &mut Vec<u32>) {
        let count = nodes
            .iter()
            .rev()
            .position(|node| self.is_node_allowed(node))
            .unwrap();
        durations.extend(iter::repeat(0).take(count));
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    macro_rules! trips {
        ($($line:ident: {$($trip:ident => $direction:ident, $times:tt);* $(;)?}),* $(,)?) => (
            $(
                pub mod $line {
                    use simulation::Direction;
                    use crate::train::*;
                    use test_utils::times;

                    $(
                        pub fn $trip(start: u32) -> Train {
                            Train {
                                direction: Direction::$direction,
                                durations: times!(start, $times),
                            }
                        }
                    )*
                }
            )*
        );
    }

    trips! {
        s3: {
            hackescher_markt_bellevue => Upstream,
            [0:30, 1:30, 0:48, 1:54, 0:36, 2:06, 0:30];
            bellevue_hackescher_markt => Downstream,
            [0:30, 2:06, 0:42, 1:54, 0:48, 1:30, 0:30];
        },
        u6: {
            naturkundemuseum_franzoesische_str => Upstream,
            [0:00, 1:30, 0:00, 1:00, 0:00, 1:30, 0:00];
            franzoesische_str_naturkundemuseum => Downstream,
            [0:00, 1:30, 0:00, 1:30, 0:00, 1:00, 0:00];
        },
        tram_m5: {
            zingster_str_perower_platz => Upstream,
            [0:00, 1:00, 0:00, 1:00, 0:00, 2:00, 0:00];
        },
        tram_12: {
            oranienburger_tor_am_kupfergraben => Upstream,
            [0:20, 2:15, 0:20, 1:05, 0:20, 1:00, 0:20];
            am_kupfergraben_oranienburger_tor => Downstream,
            [0:20, 1:15, 0:20, 1:33, 0:20, 2:32, 0:20];
        },
        bus_m82: {
            weskammstr_waldsassener_str => Upstream,
            [0:00, 0:30, 0:00, 0:30, 0:00, 1:00, 0:00];
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::trains;
    use test_utils::{time, times};

    #[test]
    fn test_time_interpolation_upstream() {
        let train = trains::tram_12::oranienburger_tor_am_kupfergraben(time!(9:01:40));
        assert_eq!(
            train.interpolate_times(simulation::fixtures::nodes::tram_12()),
            times![9:01:40, 0:20, 0:27, 1:21, 0:27, 0:20, 0:19, 0:11, 0:12, 0:12,
                0:12, 0:20, 1:00, 0:20]
        );
    }

    #[test]
    fn test_time_interpolation_downstream() {
        let train = trains::tram_12::am_kupfergraben_oranienburger_tor(time!(8:33:40));
        assert_eq!(
            train.interpolate_times(simulation::fixtures::nodes::tram_12()),
            times![8:33:40, 0:20, 0:19, 0:22, 0:12, 0:22, 0:20, 0:25, 0:12, 0:20,
                0:12, 0:24, 0:20, 0:30, 0:31, 1:01, 0:30, 0:20]
        );
    }

    #[test]
    fn test_clamp_before_dispatch() {
        let train = trains::tram_m5::zingster_str_perower_platz(time!(8:13:00));
        assert_eq!(
            train.interpolate_times(simulation::fixtures::nodes::tram_m5()),
            times!(8:13:00, 0:00, 0:00, 1:00, 0:00, 1:00, 0:00, 0:48, 1:12, 0:00)
        );
    }

    #[test]
    fn test_clamp_after_terminus() {
        let train = trains::bus_m82::weskammstr_waldsassener_str(time!(9:46:00));
        assert_eq!(
            train.interpolate_times(simulation::fixtures::nodes::bus_m82()),
            times![9:46:00, 0:00, 0:00, 0:15, 0:15, 0:00, 0:07, 0:07, 0:08, 0:07,
                0:00, 0:00, 0:00, 0:00]
        );
    }

    #[test]
    fn test_load() {
        let train = trains::tram_12::oranienburger_tor_am_kupfergraben(time!(8:13:00));
        assert_eq!(
            train.load(Kind::Tram, &simulation::fixtures::nodes::tram_12()),
            simulation::fixtures::trains::tram_12::oranienburger_tor_am_kupfergraben(
                time!(8:13:00)
            )
        );
    }
}
