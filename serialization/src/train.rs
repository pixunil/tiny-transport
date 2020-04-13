use std::iter;

use itertools::Itertools;

use serde_derive::{Serialize, Deserialize};

use simulation::{Direction, Node};
use simulation::line::Kind;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Train {
    direction: Direction,
    durations: Vec<u32>,
}

impl Train {
    pub fn new(direction: Direction, durations: Vec<u32>) -> Train {
        Train { direction, durations }
    }

    pub fn unfreeze(self, kind: Kind, nodes: &[Node]) -> simulation::Train {
        let durations = self.interpolate_times(nodes.to_vec());
        simulation::Train::new(kind, self.direction, durations)
    }

    fn interpolate_times(&self, mut nodes: Vec<Node>) -> Vec<u32> {
        if self.direction == Direction::Downstream {
            nodes.reverse();
        }

        let stop_positions = nodes.iter()
            .positions(|node| self.is_node_allowed(node));

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
        let count = nodes.iter().position(|node| self.is_node_allowed(node)).unwrap();
        durations.extend(iter::repeat(0).take(count));
    }

    fn fill_driving(&self, stop: usize, start: usize, end: usize, nodes: &[Node], durations: &mut Vec<u32>) {
        durations.push(self.durations[stop]);

        let duration = self.durations[stop + 1] as f32;

        let total_distance = self.segments_between(start, end, nodes).sum::<f32>();
        let times = self.segments_between(start, end, nodes)
            .map(|distance| {
                let travelled = distance / total_distance;
                (duration * travelled).round() as u32
            });
        durations.extend(times);
    }

    fn segments_between<'a>(&'a self, start: usize, end: usize, nodes: &'a [Node]) -> impl Iterator<Item = f32> + 'a {
        nodes[start ..= end].iter()
            .filter(move |node| node.allows(self.direction))
            .tuple_windows()
            .map(|(before, after)| na::distance(&before.position(), &after.position()))
    }

    fn fill_after_terminus(&self, nodes: &[Node], durations: &mut Vec<u32>) {
        let count = nodes.iter().rev().position(|node| self.is_node_allowed(node)).unwrap() + 1;
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
        tram_12: {
            oranienburger_tor_am_kupfergraben => Upstream, [0, 2, 0, 2, 0, 1, 0];
            am_kupfergraben_oranienburger_tor => Downstream, [0, 1, 0, 3, 0, 2, 0];
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use na::Point2;

    use simulation::{NodeKind, Directions};

    #[test]
    fn test_time_interpolation_upstream() {
        let nodes = vec![
            Node::new(Point2::new(13.37, 52.52), NodeKind::Stop, Directions::Both),
            Node::new(Point2::new(13.38, 52.52), NodeKind::Waypoint, Directions::Both),
            Node::new(Point2::new(13.38, 52.53), NodeKind::Stop, Directions::Both),
            Node::new(Point2::new(13.395, 52.53), NodeKind::Waypoint, Directions::Both),
            Node::new(Point2::new(13.395, 52.525), NodeKind::Stop, Directions::Both),
        ];

        let train = Train::new(Direction::Upstream, vec![10, 0, 4, 1, 4, 0]);
        let durations = train.interpolate_times(nodes);
        assert_eq!(durations, vec![10, 0, 2, 2, 1, 3, 1, 0]);
    }

    #[test]
    fn test_time_interpolation_downstream() {
        let nodes = vec![
            Node::new(Point2::new(13.37, 52.52), NodeKind::Stop, Directions::Both),
            Node::new(Point2::new(13.38, 52.52), NodeKind::Waypoint, Directions::Both),
            Node::new(Point2::new(13.38, 52.53), NodeKind::Stop, Directions::Both),
            Node::new(Point2::new(13.395, 52.53), NodeKind::Waypoint, Directions::Both),
            Node::new(Point2::new(13.395, 52.525), NodeKind::Stop, Directions::Both),
        ];

        let train = Train::new(Direction::Downstream, vec![10, 0, 4, 1, 4, 0]);
        let duration = train.interpolate_times(nodes);
        assert_eq!(duration, vec![10, 0, 1, 3, 1, 2, 2, 0]);
    }

    #[test]
    fn test_multiple_waypoints_on_segment() {
        let nodes = vec![
            Node::new(Point2::new(13.37, 52.52), NodeKind::Stop, Directions::Both),
            Node::new(Point2::new(13.372, 52.52), NodeKind::Waypoint, Directions::Both),
            Node::new(Point2::new(13.376, 52.52), NodeKind::Waypoint, Directions::Both),
            Node::new(Point2::new(13.382, 52.52), NodeKind::Waypoint, Directions::Both),
            Node::new(Point2::new(13.39, 52.52), NodeKind::Stop, Directions::Both),
        ];

        let train = Train::new(Direction::Upstream, vec![10, 0, 10]);
        let duration = train.interpolate_times(nodes);
        assert_eq!(duration, vec![10, 0, 1, 2, 3, 4, 0]);
    }

    #[test]
    fn test_clamp_before_dispatch() {
        let nodes = vec![
            Node::new(Point2::new(13.37, 52.515), NodeKind::Waypoint, Directions::Both),
            Node::new(Point2::new(13.37, 52.52), NodeKind::Stop, Directions::Both),
            Node::new(Point2::new(13.375, 52.52), NodeKind::Stop, Directions::Both),
        ];

        let train = Train::new(Direction::Upstream, vec![10, 0, 2, 0]);
        let duration = train.interpolate_times(nodes);
        assert_eq!(duration, vec![10, 0, 0, 2, 0]);
    }

    #[test]
    fn test_clamp_after_terminus() {
        let nodes = vec![
            Node::new(Point2::new(13.37, 52.52), NodeKind::Stop, Directions::Both),
            Node::new(Point2::new(13.375, 52.52), NodeKind::Stop, Directions::Both),
            Node::new(Point2::new(13.375, 52.515), NodeKind::Waypoint, Directions::Both),
        ];

        let train = Train::new(Direction::Upstream, vec![10, 0, 2, 0]);
        let duration = train.interpolate_times(nodes);
        assert_eq!(duration, vec![10, 0, 2, 0, 0]);
    }
}
