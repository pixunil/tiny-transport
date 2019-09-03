use std::error::Error;
use std::rc::Rc;
use std::iter;
use std::collections::HashMap;

use chrono::prelude::*;
use chrono::Duration;

use na::Point2;

use simulation::LineNode;

use crate::utils::*;
use crate::service::Service;
use crate::shape::Shape;
use crate::location::Location;
use simulation::Direction;

#[derive(Debug, PartialEq)]
pub struct Route {
    pub locations: Vec<Rc<Location>>,
    shape: Rc<Shape>,
    trips: Vec<Trip>,
}

impl Route {
    fn new(locations: Vec<Rc<Location>>, shape: Rc<Shape>) -> Route {
        Route {
            locations,
            shape,
            trips: Vec::new(),
        }
    }

    pub fn num_trips_at(&self, date: NaiveDate) -> usize {
        self.trips.iter()
            .filter(|trip| trip.service.available_at(date))
            .count()
    }

    pub fn freeze_nodes(&self) -> Vec<LineNode> {
        let mut nodes = self.shape.iter()
            .map(|waypoint| {
                let x = 2000.0 * (waypoint.x - 13.5);
                let y = -4000.0 * (waypoint.y - 52.52);
                LineNode::new(Point2::new(x, y))
            })
            .collect::<Vec<_>>();

        for station in &self.locations {
            nodes.iter_mut().min_by(|a, b| {
                let a = na::distance(&station.position(), &a.position());
                let b = na::distance(&station.position(), &b.position());
                a.partial_cmp(&b).unwrap()
            }).unwrap().promote_to_stop();
        }

        nodes
    }

    pub fn freeze_trains(&self, date: NaiveDate) -> Vec<serialization::Train> {
        self.trips.iter()
            .filter(|trip| trip.service.available_at(date))
            .map(|trip| trip.freeze())
            .collect()
    }
}

#[derive(Debug, PartialEq)]
struct Trip {
    direction: Direction,
    service: Rc<Service>,
    arrivals: Vec<Duration>,
    departures: Vec<Duration>,
}

impl Trip {
    fn freeze(&self) -> serialization::Train {
        let arrivals = self.arrivals.iter()
            .map(|duration| duration.num_seconds() as u32)
            .collect();
        let departures = self.departures.iter()
            .map(|duration| duration.num_seconds() as u32)
            .collect();
        serialization::Train::new(self.direction, arrivals, departures)
    }
}

#[derive(Debug, PartialEq)]
struct TripBuf {
    line_id: usize,
    service: Rc<Service>,
    shape_id: Id,
    direction: Direction,
    locations: Vec<Rc<Location>>,
    arrivals: Vec<Duration>,
    departures: Vec<Duration>,
}

impl TripBuf {
    fn new(line_id: usize, service: Rc<Service>, shape_id: Id, direction: Direction) -> TripBuf {
        TripBuf {
            line_id,
            service,
            shape_id,
            direction,
            locations: Vec::new(),
            arrivals: Vec::new(),
            departures: Vec::new(),
        }
    }

    fn add_stop(&mut self, record: StopRecord, locations: &HashMap<Id, Rc<Location>>) {
        let location = Rc::clone(&locations[&record.stop_id]);
        self.locations.push(location);
        self.arrivals.push(record.arrival_time);
        self.departures.push(record.departure_time);
    }

    fn termini(&self) -> (Id, Id) {
        let first = self.locations.first().unwrap().id.clone();
        let last = self.locations.last().unwrap().id.clone();
        match self.direction {
            Direction::Upstream => (first, last),
            Direction::Downstream => (last, first),
        }
    }

    fn into_trip(self, shapes: &HashMap<Id, Rc<Shape>>, routes: &mut Vec<HashMap<(Id, Id), Route>>) {
        let route = routes[self.line_id].entry(self.termini())
            .or_insert_with(|| {
                let shape = Rc::clone(&shapes[&self.shape_id]);
                Route::new(self.locations.clone(), shape)
            });
        let trip = Trip {
            direction: self.direction,
            service: self.service,
            arrivals: self.arrivals,
            departures: self.departures,
        };
        route.trips.push(trip);
    }
}

pub struct Importer<'a> {
    services: &'a HashMap<Id, Rc<Service>>,
    locations: &'a HashMap<Id, Rc<Location>>,
    shapes: &'a HashMap<Id, Rc<Shape>>,
    id_mapping: &'a HashMap<Id, usize>,
    num_lines: usize,
}

impl<'a> Importer<'a> {
    pub fn new(services: &'a HashMap<Id, Rc<Service>>, locations: &'a HashMap<Id, Rc<Location>>,
        shapes: &'a HashMap<Id, Rc<Shape>>, id_mapping: &'a HashMap<Id, usize>, num_lines: usize)
        -> Importer<'a>
    {
        Importer { services, locations, shapes, id_mapping, num_lines }
    }

    fn import_trip_buffers(&self, dataset: &mut impl Dataset) -> Result<HashMap<Id, TripBuf>, Box<dyn Error>> {
        let mut buffers = HashMap::new();
        let mut reader = dataset.read_csv("trips.txt")?;
        for result in reader.deserialize() {
            let record: TripRecord = result?;
            let line_id = self.id_mapping[&record.route_id];
            let service = Rc::clone(&self.services[&record.service_id]);
            let buffer = TripBuf::new(line_id, service, record.shape_id, record.direction_id);
            buffers.insert(record.trip_id, buffer);
        }
        Ok(buffers)
    }

    fn add_trip_stops(&self, dataset: &mut impl Dataset, buffers: &mut HashMap<Id, TripBuf>) -> Result<(), Box<dyn Error>> {
        let mut reader = dataset.read_csv("stop_times.txt")?;
        for result in reader.deserialize() {
            let record: StopRecord = result?;
            buffers.get_mut(&record.trip_id).unwrap()
                .add_stop(record, &self.locations);
        }
        Ok(())
    }

    pub fn import(self, dataset: &mut impl Dataset) -> Result<Vec<Vec<Route>>, Box<dyn Error>> {
        let mut buffers = self.import_trip_buffers(dataset)?;
        self.add_trip_stops(dataset, &mut buffers)?;

        let mut routes = iter::repeat_with(HashMap::new)
            .take(self.num_lines)
            .collect();
        for (_id, buffer) in buffers {
            buffer.into_trip(&self.shapes, &mut routes);
        }

        let routes = routes.into_iter()
            .map(|line_routes| {
                line_routes.into_iter()
                    .map(|(_, route)| route)
                    .collect()
            })
            .collect();
        Ok(routes)
    }
}

#[derive(Debug, Deserialize)]
struct TripRecord {
    trip_id: Id,
    route_id: Id,
    service_id: Id,
    shape_id: Id,
    #[serde(deserialize_with = "deserialize::direction")]
    direction_id: Direction,
}

#[derive(Debug, Deserialize)]
struct StopRecord {
    trip_id: Id,
    stop_id: Id,
    #[serde(deserialize_with = "deserialize::duration")]
    arrival_time: Duration,
    #[serde(deserialize_with = "deserialize::duration")]
    departure_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{service, station};

    fn empty_trip_buffer() -> TripBuf {
        TripBuf {
            line_id: 0,
            service: Rc::new(service!(mon-fri)),
            shape_id: "1".to_string(),
            direction: Direction::Upstream,
            locations: Vec::new(),
            arrivals: Vec::new(),
            departures: Vec::new(),
        }
    }

    #[test]
    fn test_add_stops_to_trip_buffer() {
        let records = vec![
            StopRecord {
                trip_id: "1".to_string(),
                stop_id: "1".to_string(),
                arrival_time: Duration::minutes(0),
                departure_time: Duration::minutes(1),
            },
            StopRecord {
                trip_id: "1".to_string(),
                stop_id: "2".to_string(),
                arrival_time: Duration::minutes(5),
                departure_time: Duration::minutes(6),
            },
        ];
        let mut locations = HashMap::new();
        locations.insert("1".to_string(), Rc::new(station!(main_station)));
        locations.insert("2".to_string(), Rc::new(station!(museum)));

        let expected_buffer = TripBuf {
            line_id: 0,
            service: Rc::new(service!(mon-fri)),
            shape_id: "1".to_string(),
            direction: Direction::Upstream,
            locations: vec![Rc::new(station!(main_station)), Rc::new(station!(museum))],
            arrivals: vec![Duration::minutes(0), Duration::minutes(5)],
            departures: vec![Duration::minutes(1), Duration::minutes(6)],
        };

        let mut buffer = empty_trip_buffer();
        for record in records {
            buffer.add_stop(record, &locations);
        }
        assert_eq!(buffer, expected_buffer);
    }
}
