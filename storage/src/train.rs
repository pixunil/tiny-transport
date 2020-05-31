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
            .unwrap()
            + 1;
        durations.extend(iter::repeat(0).take(count));
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    macro_rules! trips {
        ($($line:ident: {$($trip:ident => $direction:ident, [$($time:expr),*]);* $(;)?}),* $(,)?) => (
            $(
                pub mod $line {
                    use simulation::Direction;
                    use crate::train::*;

                    $(
                        pub fn $trip(hour: u32, minute: f64) -> Train {
                            let start = hour * 3600 + (minute * 60.0) as u32;
                            Train {
                                direction: Direction::$direction,
                                durations: vec![
                                    start,
                                    $( ($time as f64 * 60.0) as u32 ),*
                                ],
                            }
                        }
                    )*
                }
            )*
        );
    }

    trips! {
        tram_m5: {
            zingster_str_perower_platz => Upstream, [0, 1, 0, 1, 0, 2, 0];
        },
        tram_12: {
            oranienburger_tor_am_kupfergraben => Upstream, [0, 2, 0, 2, 0, 1, 0];
            am_kupfergraben_oranienburger_tor => Downstream, [0, 1, 0, 3, 0, 2, 0];
        },
        bus_m82: {
            weskammstr_waldsassener_str => Upstream, [0, 0.5, 0, 0.5, 0, 1, 0];
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::*;
    use simulation::fixtures::nodes;

    #[test]
    fn test_time_interpolation_upstream() {
        let train = trains::tram_12::oranienburger_tor_am_kupfergraben(9, 2.0);
        let durations = train.interpolate_times(nodes::tram_12());
        assert_eq!(
            durations,
            vec![32520, 0, 24, 72, 24, 0, 35, 21, 21, 21, 21, 0, 60, 0]
        );
    }

    #[test]
    fn test_time_interpolation_downstream() {
        let train = trains::tram_12::am_kupfergraben_oranienburger_tor(8, 34.0);
        let duration = train.interpolate_times(nodes::tram_12());
        assert_eq!(
            duration,
            vec![30840, 0, 15, 18, 9, 18, 0, 48, 24, 39, 24, 46, 0, 24, 24, 48, 24, 0]
        );
    }

    #[test]
    fn test_clamp_before_dispatch() {
        let train = trains::tram_m5::zingster_str_perower_platz(8, 13.0);
        let duration = train.interpolate_times(nodes::tram_m5());
        assert_eq!(duration, vec![29580, 0, 0, 60, 0, 60, 0, 48, 72, 0]);
    }

    #[test]
    fn test_clamp_after_terminus() {
        let train = trains::bus_m82::weskammstr_waldsassener_str(9, 46.0);
        let duration = train.interpolate_times(nodes::bus_m82());
        assert_eq!(
            duration,
            vec![35160, 0, 0, 15, 15, 0, 7, 7, 8, 7, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_load() {
        let train = trains::tram_12::oranienburger_tor_am_kupfergraben(8, 13.0);
        assert_eq!(
            train.load(Kind::Tram, &nodes::tram_12()),
            simulation::fixtures::trains::tram_12::oranienburger_tor_am_kupfergraben(8, 13.0)
        );
    }
}
