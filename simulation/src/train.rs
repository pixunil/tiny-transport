use serde_derive::{Serialize, Deserialize};
use na::{Vector2, Matrix2};

use crate::line::LineNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Upstream,
    Downstream,
}

impl Direction {
    fn track(self, nodes: &[LineNode], current: usize) -> (&LineNode, &LineNode) {
        let len = nodes.len();
        match self {
            Direction::Upstream => (&nodes[current - 1], &nodes[current]),
            Direction::Downstream => (&nodes[len - current], &nodes[len - current - 1]),
        }
    }
}

#[derive(Debug)]
pub struct Train {
    direction: Direction,
    pub arrivals: Vec<u32>,
    pub departures: Vec<u32>,
    current: usize,
    travelled: f32,
}

impl Train {
    pub fn new(direction: Direction, arrivals: Vec<u32>, departures: Vec<u32>) -> Train {
        Train {
            direction,
            arrivals,
            departures,
            current: 0,
            travelled: 0.0,
        }
    }

    pub fn update(&mut self, time: u32) {
        while self.current < self.arrivals.len() && self.arrivals[self.current] < time {
            self.current += 1;
        }

        if self.is_active() {
            let travelled_time = time.checked_sub(self.departures[self.current - 1]).unwrap_or(0);
            let travel_time = self.arrivals[self.current] - self.departures[self.current - 1];
            self.travelled = travelled_time as f32 / travel_time as f32;
        }
    }

    pub fn is_active(&self) -> bool {
        0 < self.current && self.current < self.arrivals.len()
    }

    pub fn fill_vertice_buffer(&self, buffer: &mut Vec<f32>, nodes: &[LineNode]) {
        let (current, next) = self.direction.track(nodes, self.current);
        let direction = next.position() - current.position();
        let position = current.position() + direction * self.travelled;
        let orientation = direction.normalize();
        let bounds = Matrix2::from_columns(&[orientation, Vector2::new(-orientation.y, orientation.x)]);

        let right_front = position + bounds * Vector2::new(4.5, 3.0);
        let left_front = position + bounds * Vector2::new(4.5, -3.0);
        let right_back = position + bounds * Vector2::new(-4.5, 3.0);
        let left_back = position + bounds * Vector2::new(-4.5, -3.0);
        buffer.extend(left_back.iter().chain(left_front.iter()).chain(right_back.iter()));
        buffer.extend(right_front.iter().chain(right_back.iter()).chain(left_front.iter()));
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    #[test]
    fn test_before_dispatch() {
        let mut train = Train::new(Direction::Upstream, vec![1, 4, 7], vec![2, 6, 8]);
        train.update(0);
        assert!(!train.is_active());
    }

    #[test]
    fn test_stopped() {
        let mut train = Train::new(Direction::Upstream, vec![1, 4, 7], vec![2, 6, 8]);
        train.update(2);
        assert!(train.is_active());

        let mut buffer = Vec::new();
        train.fill_vertice_buffer(&mut buffer, &crate::line_nodes!(blue));
        assert_relative_eq!(*buffer, [
            195.5, 97.0,
            204.5, 97.0,
            195.5, 103.0,
            204.5, 103.0,
            195.5, 103.0,
            204.5, 97.0,
        ]);
    }

    #[test]
    fn test_driving() {
        let mut train = Train::new(Direction::Upstream, vec![1, 4, 7], vec![2, 6, 8]);
        train.update(3);
        assert!(train.is_active());

        let mut buffer = Vec::new();
        train.fill_vertice_buffer(&mut buffer, &crate::line_nodes!(blue));
        assert_relative_eq!(*buffer, [
            205.5, 97.0,
            214.5, 97.0,
            205.5, 103.0,
            214.5, 103.0,
            205.5, 103.0,
            214.5, 97.0,
        ]);
    }

    #[test]
    fn test_after_terminus() {
        let mut train = Train::new(Direction::Upstream, vec![1, 4, 7], vec![2, 6, 8]);
        train.update(8);
        assert!(!train.is_active());
    }
}
