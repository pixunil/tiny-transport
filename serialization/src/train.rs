use std::iter;

use serde_derive::{Serialize, Deserialize};

use simulation::{Direction, LineNode};

#[derive(Debug, Serialize, Deserialize)]
pub struct Train {
    direction: Direction,
    durations: Vec<u32>,
}

impl Train {
    pub fn new(direction: Direction, durations: Vec<u32>) -> Train {
        Train { direction, durations }
    }

    pub fn unfreeze(self, nodes: &[LineNode]) -> simulation::Train {
        let durations = self.interpolate_times(nodes.to_vec());
        simulation::Train::new(self.direction, durations)
    }

    fn interpolate_times(&self, mut nodes: Vec<LineNode>) -> Vec<u32> {
        if self.direction == Direction::Downstream {
            nodes.reverse();
        }

        let stop_positions = nodes.iter()
            .enumerate()
            .filter(|(_, node)| node.is_stop())
            .map(|(position, _)| position)
            .collect::<Vec<_>>();

        let mut durations = vec![self.durations[0]];
        self.fill_before_dispatch(&nodes, &mut durations);

        for (i, segment) in stop_positions.windows(2).enumerate() {
            self.fill_driving(2 * i + 1, segment[0], segment[1], &nodes, &mut durations);
        }

        self.fill_after_terminus(&nodes, &mut durations);
        durations
    }

    fn fill_before_dispatch(&self, nodes: &[LineNode], durations: &mut Vec<u32>) {
        let count = nodes.iter().position(LineNode::is_stop).unwrap();
        durations.extend(iter::repeat(0).take(count));
    }

    fn fill_driving(&self, stop: usize, start: usize, end: usize, nodes: &[LineNode], durations: &mut Vec<u32>) {
        durations.push(self.durations[stop]);

        let duration = self.durations[stop + 1] as f32;

        let distances = nodes[start ..= end].windows(2)
            .map(|segment| na::distance(&segment[0].position(), &segment[1].position()));
        let total_distance = distances.clone().sum::<f32>();

        for distance in distances {
            let travelled = distance / total_distance;
            let time = (duration * travelled).round() as u32;
            durations.push(time);
        }
    }

    fn fill_after_terminus(&self, nodes: &[LineNode], durations: &mut Vec<u32>) {
        let count = nodes.iter().rev().position(LineNode::is_stop).unwrap() + 1;
        durations.extend(iter::repeat(0).take(count));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use na::Point2;

    #[test]
    fn test_time_interpolation_upstream() {
        let mut nodes = vec![
            LineNode::new(Point2::new(13.37, 52.52)),
            LineNode::new(Point2::new(13.38, 52.52)),
            LineNode::new(Point2::new(13.38, 52.53)),
            LineNode::new(Point2::new(13.395, 52.53)),
            LineNode::new(Point2::new(13.395, 52.525)),
        ];

        nodes[0].promote_to_stop();
        nodes[2].promote_to_stop();
        nodes[4].promote_to_stop();

        let train = Train::new(Direction::Upstream, vec![10, 0, 4, 1, 4, 0]);
        let durations = train.interpolate_times(nodes);
        assert_eq!(durations, vec![10, 0, 2, 2, 1, 3, 1, 0]);
    }

    #[test]
    fn test_time_interpolation_downstream() {
        let mut nodes = vec![
            LineNode::new(Point2::new(13.37, 52.52)),
            LineNode::new(Point2::new(13.38, 52.52)),
            LineNode::new(Point2::new(13.38, 52.53)),
            LineNode::new(Point2::new(13.395, 52.53)),
            LineNode::new(Point2::new(13.395, 52.525)),
        ];

        nodes[0].promote_to_stop();
        nodes[2].promote_to_stop();
        nodes[4].promote_to_stop();

        let train = Train::new(Direction::Downstream, vec![10, 0, 4, 1, 4, 0]);
        let duration = train.interpolate_times(nodes);
        assert_eq!(duration, vec![10, 0, 1, 3, 1, 2, 2, 0]);
    }

    #[test]
    fn test_multiple_waypoints_on_segment() {
        let mut nodes = vec![
            LineNode::new(Point2::new(13.37, 52.52)),
            LineNode::new(Point2::new(13.372, 52.52)),
            LineNode::new(Point2::new(13.376, 52.52)),
            LineNode::new(Point2::new(13.382, 52.52)),
            LineNode::new(Point2::new(13.39, 52.52)),
        ];

        nodes[0].promote_to_stop();
        nodes[4].promote_to_stop();

        let train = Train::new(Direction::Upstream, vec![10, 0, 10]);
        let duration = train.interpolate_times(nodes);
        assert_eq!(duration, vec![10, 0, 1, 2, 3, 4, 0]);
    }

    #[test]
    fn test_clamp_before_dispatch() {
        let mut nodes = vec![
            LineNode::new(Point2::new(13.37, 52.515)),
            LineNode::new(Point2::new(13.37, 52.52)),
            LineNode::new(Point2::new(13.375, 52.52)),
        ];

        nodes[1].promote_to_stop();
        nodes[2].promote_to_stop();

        let train = Train::new(Direction::Upstream, vec![10, 0, 2, 0]);
        let duration = train.interpolate_times(nodes);
        assert_eq!(duration, vec![10, 0, 0, 2, 0]);
    }

    #[test]
    fn test_clamp_after_terminus() {
        let mut nodes = vec![
            LineNode::new(Point2::new(13.37, 52.52)),
            LineNode::new(Point2::new(13.375, 52.52)),
            LineNode::new(Point2::new(13.375, 52.515)),
        ];

        nodes[0].promote_to_stop();
        nodes[1].promote_to_stop();

        let train = Train::new(Direction::Upstream, vec![10, 0, 2, 0]);
        let duration = train.interpolate_times(nodes);
        assert_eq!(duration, vec![10, 0, 2, 0, 0]);
    }
}
