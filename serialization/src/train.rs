use std::iter;

use serde_derive::{Serialize, Deserialize};

use simulation::{Direction, LineNode};

#[derive(Debug, Serialize, Deserialize)]
pub struct Train {
    direction: Direction,
    arrivals: Vec<u32>,
    departures: Vec<u32>,
}

impl Train {
    pub fn new(direction: Direction, arrivals: Vec<u32>, departures: Vec<u32>) -> Train {
        Train { direction, arrivals, departures }
    }

    pub fn unfreeze(self, nodes: &[LineNode]) -> simulation::Train {
        let (arrivals, departures) = self.interpolate_times(nodes.to_vec());
        simulation::Train::new(self.direction, arrivals, departures)
    }

    fn interpolate_times(&self, mut nodes: Vec<LineNode>) -> (Vec<u32>, Vec<u32>) {
        if self.direction == Direction::Downstream {
            nodes.reverse();
        }

        let stop_positions = nodes.iter()
            .enumerate()
            .filter(|(_, node)| node.is_stop())
            .map(|(position, _)| position)
            .collect::<Vec<_>>();

        let (mut arrivals, mut departures) = self.fill_before_dispatch(&stop_positions);

        for (stop, segment) in stop_positions.windows(2).enumerate() {
            self.fill_driving(stop, segment[0], segment[1], &nodes, &mut arrivals, &mut departures);
        }

        self.fill_after_terminus(&nodes, &mut arrivals, &mut departures);
        (arrivals, departures)
    }

    fn fill_before_dispatch(&self, stop_positions: &[usize]) -> (Vec<u32>, Vec<u32>) {
        let arrivals = iter::repeat(self.arrivals[0])
            .take(stop_positions[0])
            .collect::<Vec<_>>();
        let departures = iter::repeat(self.arrivals[0])
            .take(stop_positions[0])
            .collect::<Vec<_>>();
        (arrivals, departures)
    }

    fn fill_driving(&self, stop: usize, start: usize, end: usize, nodes: &[LineNode], arrivals: &mut Vec<u32>, departures: &mut Vec<u32>) {
        arrivals.push(self.arrivals[stop]);
        departures.push(self.departures[stop]);

        let departure = self.departures[stop] as f32;
        let arrival = self.arrivals[stop + 1] as f32;

        let mut distances = nodes[start ..= end].windows(2)
            .scan(0.0, |distance, segment| {
                *distance += na::distance(&segment[0].position(), &segment[1].position());
                Some(*distance)
            })
            .collect::<Vec<_>>();
        let total_distance = distances.pop().unwrap();

        for distance in distances {
            let travelled = distance / total_distance;
            let time = (departure * (1.0 - travelled) + arrival * travelled).round() as u32;
            arrivals.push(time);
            departures.push(time);
        }
    }

    fn fill_after_terminus(&self, nodes: &[LineNode], arrivals: &mut Vec<u32>, departures: &mut Vec<u32>) {
        let departure = self.departures.last().unwrap();
        let count = nodes.iter().rev().position(LineNode::is_stop).unwrap() + 1;
        arrivals.extend(iter::repeat(departure).take(count));
        departures.extend(iter::repeat(departure).take(count));
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

        let train = Train::new(Direction::Upstream, vec![0, 4, 9], vec![0, 5, 9]);
        let (arrivals, departures) = train.interpolate_times(nodes);
        assert_eq!(arrivals, vec![0, 2, 4, 8, 9]);
        assert_eq!(departures, vec![0, 2, 5, 8, 9]);
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

        let train = Train::new(Direction::Downstream, vec![0, 4, 9], vec![0, 5, 9]);
        let (arrivals, departures) = train.interpolate_times(nodes);
        assert_eq!(arrivals, vec![0, 1, 4, 7, 9]);
        assert_eq!(departures, vec![0, 1, 5, 7, 9]);
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

        let train = Train::new(Direction::Upstream, vec![0, 10], vec![0, 10]);
        let (arrivals, departures) = train.interpolate_times(nodes);
        assert_eq!(arrivals, vec![0, 1, 3, 6, 10]);
        assert_eq!(departures, vec![0, 1, 3, 6, 10]);
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

        let train = Train::new(Direction::Upstream, vec![0, 2], vec![0, 2]);
        let (arrivals, departures) = train.interpolate_times(nodes);
        assert_eq!(arrivals, vec![0, 0, 2]);
        assert_eq!(departures, vec![0, 0, 2]);
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

        let train = Train::new(Direction::Upstream, vec![0, 2], vec![0, 2]);
        let (arrivals, departures) = train.interpolate_times(nodes);
        assert_eq!(arrivals, vec![0, 2, 2]);
        assert_eq!(departures, vec![0, 2, 2]);
    }
}
