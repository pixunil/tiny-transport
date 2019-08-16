use std::error::Error;
use std::rc::Rc;
use std::iter;
use std::collections::HashMap;

use chrono::prelude::*;
use chrono::Duration;

use na::Point2;

use simulation::LineNode;

use super::utils::*;
use super::service::Service;
use super::shape::Shape;
use super::location::{Location, Path};

#[derive(Debug, PartialEq)]
pub struct Route {
    pub locations: Vec<Rc<Location>>,
    shape: Rc<Shape>,
    trips: Vec<Trip>,
}

impl Route {
    fn new(locations: Vec<Rc<Location>>, shape: Rc<Shape>, trips: Vec<Trip>) -> Route {
        Route { locations, shape, trips }
    }

    pub fn num_trips_at(&self, date: &NaiveDate) -> usize {
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

    pub fn freeze_trains(&self, date: &NaiveDate) -> Vec<serialization::Train> {
        self.trips.iter()
            .filter(|trip| trip.service.available_at(date))
            .map(|trip| trip.freeze())
            .collect()
    }
}

#[derive(Debug, PartialEq)]
struct Trip {
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
        serialization::Train::new(arrivals, departures)
    }
}

#[derive(Debug, PartialEq)]
struct TripBuf {
    line_id: usize,
    service: Rc<Service>,
    shape_id: Id,
    locations: Vec<Rc<Location>>,
    arrivals: Vec<Duration>,
    departures: Vec<Duration>,
}

impl TripBuf {
    fn new(record: TripRecord, services: &HashMap<Id, Rc<Service>>, id_mapping: &HashMap<Id, usize>) -> (Id, TripBuf) {
        let trip = TripBuf {
            line_id: id_mapping[&record.route_id],
            service: services[&record.service_id].clone(),
            shape_id: record.shape_id,
            locations: Vec::new(),
            arrivals: Vec::new(),
            departures: Vec::new(),
        };
        (record.trip_id, trip)
    }

    fn add_stop(&mut self, record: StopRecord, locations: &HashMap<Id, Rc<Location>>) {
        self.locations.push(locations[&record.stop_id].clone());
        self.arrivals.push(record.arrival_time);
        self.departures.push(record.departure_time);
    }

    fn into_trip(self, trips: &mut Vec<HashMap<(Id, Path), Vec<Trip>>>) {
        let path = Path::new(self.locations);

        let trip = Trip {
            service: self.service,
            arrivals: self.arrivals,
            departures: self.departures,
        };

        trips[self.line_id].entry((self.shape_id, path))
            .or_insert_with(Vec::new)
            .push(trip);
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
            let (id, buffer) = TripBuf::new(result?, &self.services,  &self.id_mapping);
            buffers.insert(id, buffer);
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

    pub fn import(self, dataset: &mut impl Dataset) -> Result<Vec<Vec<Route>>, Box<dyn Error>>
    {
        let mut buffers = self.import_trip_buffers(dataset)?;
        self.add_trip_stops(dataset, &mut buffers)?;

        let mut trips = iter::repeat_with(HashMap::new)
            .take(self.num_lines)
            .collect();
        for (_id, buffer) in buffers {
            buffer.into_trip(&mut trips);
        }

        let routes = trips.into_iter()
            .map(|routes| {
                routes.into_iter()
                .map(|((shape_id, path), trips)| Route::new(path.into(), self.shapes[&shape_id].clone(), trips))
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
}

#[derive(Debug, Deserialize)]
struct StopRecord {
    trip_id: Id,
    stop_id: Id,
    #[serde(deserialize_with = "deserialize_duration")]
    arrival_time: Duration,
    #[serde(deserialize_with = "deserialize_duration")]
    departure_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::service::tests::service_monday_to_friday;
    use crate::location::tests::{main_station, museum};

    fn empty_trip_buffer() -> TripBuf {
        TripBuf {
            line_id: 0,
            service: Rc::new(service_monday_to_friday()),
            shape_id: "1".into(),
            locations: Vec::new(),
            arrivals: Vec::new(),
            departures: Vec::new(),
        }
    }

    #[test]
    fn test_import_trip_buffer() {
        let mut services = HashMap::new();
        services.insert("1".into(), Rc::new(service_monday_to_friday()));
        let mut id_mapping = HashMap::new();
        id_mapping.insert("1".into(), 0);
        let record = TripRecord {
            trip_id: "1".into(),
            route_id: "1".into(),
            service_id: "1".into(),
            shape_id: "1".into(),
        };
        assert_eq!(TripBuf::new(record, &services, &id_mapping), ("1".into(), empty_trip_buffer()));
    }

    #[test]
    fn test_add_stops_to_trip_buffer() {
        let records = vec![
            StopRecord {
                trip_id: "1".into(),
                stop_id: "1".into(),
                arrival_time: Duration::minutes(0),
                departure_time: Duration::minutes(1),
            },
            StopRecord {
                trip_id: "1".into(),
                stop_id: "2".into(),
                arrival_time: Duration::minutes(5),
                departure_time: Duration::minutes(6),
            },
        ];
        let mut locations = HashMap::new();
        locations.insert("1".into(), Rc::new(main_station()));
        locations.insert("2".into(), Rc::new(museum()));

        let expected_buffer = TripBuf {
            line_id: 0,
            service: Rc::new(service_monday_to_friday()),
            shape_id: "1".into(),
            locations: vec![Rc::new(main_station()), Rc::new(museum())],
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
