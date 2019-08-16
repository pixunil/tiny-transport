use std::iter;

use simulation::LineNode;

#[derive(Debug, Serialize, Deserialize)]
pub struct Train {
    arrivals: Vec<u32>,
    departures: Vec<u32>,
}

impl Train {
    pub fn new(arrivals: Vec<u32>, departures: Vec<u32>) -> Train {
        Train {
            arrivals,
            departures,
        }
    }

    pub fn unfreeze(self, nodes: &[LineNode]) -> simulation::Train {
        let stops = nodes.iter()
            .enumerate()
            .filter(|(_, node)| node.is_stop())
            .map(|(position, _)| position)
            .collect::<Vec<_>>();

        let mut arrivals = iter::repeat(self.arrivals[0])
            .take(stops[0])
            .collect::<Vec<_>>();
        let mut departures = iter::repeat(self.departures[0])
            .take(stops[0])
            .collect::<Vec<_>>();

        for (i, segment) in stops.windows(2).enumerate() {
            arrivals.push(self.arrivals[i]);
            departures.push(self.departures[i]);

            let departure = self.departures[i] as f32;
            let arrival = self.arrivals[i + 1] as f32;

            let start = segment[0];
            let end = segment[1];

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

        arrivals.push(*self.arrivals.last().unwrap());
        departures.push(*self.departures.last().unwrap());

        simulation::Train::new(arrivals, departures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use na::Point2;

    #[test]
    fn test_time_interpolation() {
        let train = Train {
            arrivals: vec![0, 5, 10],
            departures: vec![0, 6, 10],
        };

        let mut nodes = [
            LineNode::new(Point2::new(13.37, 52.52)),
            LineNode::new(Point2::new(13.37, 52.53)),
            LineNode::new(Point2::new(13.38, 52.53)),
            LineNode::new(Point2::new(13.39, 52.53)),
            LineNode::new(Point2::new(13.40, 52.54)),
        ];

        nodes[0].promote_to_stop();
        nodes[2].promote_to_stop();
        nodes[4].promote_to_stop();

        let train = train.unfreeze(&nodes);
        assert_eq!(train.arrivals, vec![0, 2, 5, 8, 10]);
        assert_eq!(train.departures, vec![0, 2, 6, 8, 10]);
    }

    #[test]
    fn test_multiple_waypoints_on_segment() {
        let train = Train {
            arrivals: vec![0, 10],
            departures: vec![0, 10],
        };

        let mut nodes = [
            LineNode::new(Point2::new(13.37, 52.52)),
            LineNode::new(Point2::new(13.372, 52.52)),
            LineNode::new(Point2::new(13.376, 52.52)),
            LineNode::new(Point2::new(13.382, 52.52)),
            LineNode::new(Point2::new(13.39, 52.52)),
        ];

        nodes[0].promote_to_stop();
        nodes[4].promote_to_stop();

        let train = train.unfreeze(&nodes);
        assert_eq!(train.arrivals, vec![0, 1, 3, 6, 10]);
        assert_eq!(train.departures, vec![0, 1, 3, 6, 10]);
    }
}