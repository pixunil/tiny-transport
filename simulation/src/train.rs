use na::{Vector2, Matrix2};

use crate::line::LineNode;

#[derive(Debug)]
pub struct Train {
    pub arrivals: Vec<u32>,
    pub departures: Vec<u32>,
    current: usize,
    travelled: f32,
}

impl Train {
    pub fn new(arrivals: Vec<u32>, departures: Vec<u32>) -> Train {
        Train {
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
        let current = &nodes[self.current - 1];
        let next = &nodes[self.current];
        let direction = next.position() - current.position();
        let position = current.position() + direction * self.travelled;
        let orientation = direction.normalize();
        let bounds = Matrix2::from_columns(&[orientation, Vector2::new(-orientation.y, orientation.x)]);

        let right_front = position + bounds * Vector2::new(4.5, 3.0);
        let left_front = position + bounds * Vector2::new(-4.5, 3.0);
        let right_back = position + bounds * Vector2::new(4.5, -3.0);
        let left_back = position + bounds * Vector2::new(-4.5, -3.0);
        buffer.extend(left_back.iter().chain(left_front.iter()).chain(right_back.iter()));
        buffer.extend(right_front.iter().chain(right_back.iter()).chain(left_front.iter()));
    }
}
