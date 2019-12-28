use std::rc::Rc;
use std::collections::HashMap;

use chrono::Duration;

use simulation::Direction;
use crate::create_id_type;
use crate::service::Service;
use crate::shape::{Shape, ShapeId};
use crate::location::{Location, LocationId};
use super::{Trip, Route};

create_id_type!(TripId);

#[derive(Debug, PartialEq)]
pub(super) struct TripBuffer {
    line_id: usize,
    service: Rc<Service>,
    shape_id: ShapeId,
    direction: Direction,
    locations: Vec<Rc<Location>>,
    arrivals: Vec<Duration>,
    departures: Vec<Duration>,
}

impl TripBuffer {
    pub(super) fn new(line_id: usize, service: Rc<Service>, shape_id: ShapeId, direction: Direction) -> TripBuffer {
        TripBuffer {
            line_id,
            service,
            shape_id,
            direction,
            locations: Vec::new(),
            arrivals: Vec::new(),
            departures: Vec::new(),
        }
    }

    pub(super) fn add_stop(&mut self, location: Rc<Location>, arrival: Duration, departure: Duration) {
        self.locations.push(location);
        self.arrivals.push(arrival);
        self.departures.push(departure);
    }

    fn termini(&self) -> (LocationId, LocationId) {
        let first = self.locations.first().unwrap().id.clone();
        let last = self.locations.last().unwrap().id.clone();
        match self.direction {
            Direction::Upstream => (first, last),
            Direction::Downstream => (last, first),
        }
    }

    fn into_trip(self) -> Trip {
        let mut durations = Vec::new();
        for (i, &arrival) in self.arrivals.iter().enumerate() {
            if i == 0 {
                durations.push(arrival);
            } else {
                durations.push(arrival - self.departures[i - 1]);
            }
            durations.push(self.departures[i] - arrival);
        }

        Trip::new(self.direction, self.service, durations)
    }

    pub(super) fn place_into_routes(self, shapes: &HashMap<ShapeId, Shape>, routes: &mut Vec<HashMap<(LocationId, LocationId), Route>>) {
        let route = routes[self.line_id].entry(self.termini())
            .or_insert_with(|| {
                let mut locations = self.locations.clone();
                let mut shape = shapes[&self.shape_id].clone();
                if self.direction == Direction::Downstream {
                    locations.reverse();
                    shape.reverse();
                }
                Route::new(locations, shape)
            });
        route.add_trip(self.into_trip());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{service, station};

    fn empty_trip_buffer() -> TripBuffer {
        TripBuffer {
            line_id: 0,
            service: Rc::new(service!(mon-fri)),
            shape_id: "1".into(),
            direction: Direction::Upstream,
            locations: Vec::new(),
            arrivals: Vec::new(),
            departures: Vec::new(),
        }
    }

    fn completed_trip_buffer() -> TripBuffer {
        TripBuffer {
            line_id: 0,
            service: Rc::new(service!(mon-fri)),
            shape_id: "1".into(),
            direction: Direction::Upstream,
            locations: station![main_station, center, market],
            arrivals: vec![Duration::minutes(1), Duration::minutes(5), Duration::minutes(10)],
            departures: vec![Duration::minutes(1), Duration::minutes(6), Duration::minutes(10)],
        }
    }

    #[test]
    fn test_add_stop() {
        let mut buffer = empty_trip_buffer();
        buffer.add_stop(Rc::new(station!(main_station)), Duration::minutes(1), Duration::minutes(1));
        buffer.add_stop(Rc::new(station!(center)), Duration::minutes(5), Duration::minutes(6));
        buffer.add_stop(Rc::new(station!(market)), Duration::minutes(10), Duration::minutes(10));
        assert_eq!(buffer, completed_trip_buffer());
    }

    #[test]
    fn test_into_trip() {
        let buffer = completed_trip_buffer();
        let trip = Trip::new(
            Direction::Upstream,
            Rc::new(service!(mon-fri)),
            vec![Duration::minutes(1), Duration::minutes(0), Duration::minutes(4),
                 Duration::minutes(1), Duration::minutes(4), Duration::minutes(0)]);
        assert_eq!(buffer.into_trip(), trip);
    }
}
