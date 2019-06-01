use std::convert::{TryFrom, TryInto};
use std::collections::HashMap;

use na::{Vector2, Matrix2};

use crate::track::{Connection, Track, TrackBundle};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Direction {
    Upstream,
    Downstream,
}

impl Direction {
    fn tracks(self, tracks: &[Track], current: usize) -> (Option<&Track>, &Track, Option<&Track>) {
        let index = match self {
            Direction::Upstream => current,
            Direction::Downstream => tracks.len() - 1 - current,
        };

        (
            index.checked_sub(1).map(|index| &tracks[index]),
            &tracks[index],
            tracks.get(index + 1),
        )
    }

    fn interpolation(self, travelled: f32) -> f32 {
        match self {
            Direction::Upstream => travelled,
            Direction::Downstream => 1.0 - travelled,
        }
    }
}

impl<'a> TryFrom<&'a str> for Direction {
    type Error = &'a str;

    fn try_from(value: &str) -> Result<Direction, &str> {
        match value {
            "upstream" => Ok(Direction::Upstream),
            "downstream" => Ok(Direction::Downstream),
            value => Err(value),
        }
    }
}

#[derive(Debug)]
pub struct Train {
    direction: Direction,
    arrivals: Vec<u32>,
    departures: Vec<u32>,
    current: usize,
    travelled: f32,
}

impl Train {
    fn new(direction: Direction, arrivals: Vec<u32>, departures: Vec<u32>) -> Train {
        Train {
            direction,
            arrivals,
            departures,
            current: 0,
            travelled: 0.0,
        }
    }

    pub fn from_json(json: &serde_json::Value) -> Train {
        let direction = json["direction"].as_str().unwrap().try_into().unwrap();
        let arrivals = json["arrivals"].as_array().unwrap()
            .iter()
            .map(|time| time.as_u64().unwrap() as u32)
            .collect();
        let departures = json["departures"].as_array().unwrap()
            .iter()
            .map(|time| time.as_u64().unwrap() as u32)
            .collect();
        Train::new(direction, arrivals, departures)
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

    pub fn fill_vertice_buffer(&self, buffer: &mut Vec<f32>, tracks: &[Track], track_bundles: &HashMap<Connection, TrackBundle>) {
        let (preceding, track, following) = self.direction.tracks(tracks, self.current - 1);
        let interpolation = self.direction.interpolation(self.travelled);
        let (position, orientation) = track.interpolated_position(preceding, following, track_bundles, interpolation);
        let bounds = Matrix2::from_columns(&[orientation, Vector2::new(-orientation.y, orientation.x)]);

        let right_front = position + bounds * Vector2::new(4.5, 3.0);
        let left_front = position + bounds * Vector2::new(-4.5, 3.0);
        let right_back = position + bounds * Vector2::new(4.5, -3.0);
        let left_back = position + bounds * Vector2::new(-4.5, -3.0);
        buffer.extend(left_back.iter());
        buffer.extend(left_front.iter());
        buffer.extend(right_back.iter());
        buffer.extend(right_front.iter());
        buffer.extend(right_back.iter());
        buffer.extend(left_front.iter());
    }
}
