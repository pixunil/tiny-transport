use std::error::Error;
use std::rc::Rc;
use std::collections::HashMap;
use std::path::PathBuf;

use chrono::prelude::*;
use chrono::Duration;

use simulation::Direction;

use super::utils::*;
use super::service::Service;
use super::location::Location;

#[derive(Debug)]
pub struct Route {
    pub locations: Vec<Rc<Location>>,
    trips: Vec<Trip>,
}

impl Route {
    fn new(locations: Vec<Rc<Location>>, trips: Vec<Trip>) -> Route {
        Route {
            locations,
            trips,
        }
    }

    pub fn num_trips_at(&self, date: &NaiveDate) -> usize {
        self.trips.iter()
            .filter(|trip| trip.service.available_at(date))
            .count()
    }

    pub fn into_stops(&self, stations: &[Rc<Location>]) -> Vec<usize> {
        self.locations.iter()
            .map(|location| {
                stations.iter()
                    .position(|station| location == station)
                    .unwrap()
            })
            .collect()
    }

    pub fn into_trains(&self, date: &NaiveDate) -> Vec<simulation::Train> {
        self.trips.iter()
            .filter(|trip| trip.service.available_at(date))
            .map(|trip| trip.into_train())
            .collect()
    }
}

#[derive(Debug)]
struct Trip {
    direction: Direction,
    service: Rc<Service>,
    arrivals: Vec<Duration>,
    departures: Vec<Duration>,
}

impl Trip {
    fn into_train(&self) -> simulation::Train {
        let arrivals = self.arrivals.iter()
            .map(|duration| duration.num_seconds() as u32)
            .collect();
        let departures = self.departures.iter()
            .map(|duration| duration.num_seconds() as u32)
            .collect();
        simulation::Train::new(self.direction, arrivals, departures)
    }
}

#[derive(Debug)]
struct TripBuf {
    line_id: Id,
    service: Rc<Service>,
    locations: Vec<Rc<Location>>,
    arrivals: Vec<Duration>,
    departures: Vec<Duration>,
}

impl TripBuf {
    fn new(record: TripRecord, services: &HashMap<Id, Rc<Service>>) -> (Id, TripBuf) {
        let trip = TripBuf {
            line_id: record.route_id,
            service: services[&record.service_id].clone(),
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

    fn into_trip(self, trips: &mut HashMap<(Id, Vec<Rc<Location>>), Vec<Trip>>) {
        let mut locations = self.locations.clone();
        let direction = if locations.first() <= locations.last() {
            Direction::Upstream
        } else {
            locations.reverse();
            Direction::Downstream
        };

        let trip = Trip {
            direction,
            service: self.service,
            arrivals: self.arrivals,
            departures: self.departures,
        };

        trips.entry((self.line_id, locations))
            .or_insert_with(Vec::new)
            .push(trip);
    }
}

pub fn from_csv(path: &mut PathBuf, services: &HashMap<Id, Rc<Service>>, locations: &HashMap<Id, Rc<Location>>)
    -> Result<HashMap<Id, Vec<Route>>, Box<Error>>
{
    let mut buffers = HashMap::new();

    path.set_file_name("trips.txt");
    let mut reader = csv::Reader::from_path(&path)?;
    for result in reader.deserialize() {
        let (id, buffer) = TripBuf::new(result?, services);
        buffers.insert(id, buffer);
    }

    path.set_file_name("stop_times.txt");
    let mut reader = csv::Reader::from_path(&path)?;
    for result in reader.deserialize() {
        let record: StopRecord = result?;
        buffers.get_mut(&record.trip_id).unwrap()
            .add_stop(record, locations);
    }

    let mut trips = HashMap::new();
    for (_id, buffer) in buffers {
        buffer.into_trip(&mut trips);
    }

    let mut routes = HashMap::new();
    for ((line_id, locations), trips) in trips {
        routes.entry(line_id)
            .or_insert_with(Vec::new)
            .push(Route::new(locations, trips))
    }

    Ok(routes)
}

#[derive(Debug, Deserialize)]
struct TripRecord {
    trip_id: Id,
    route_id: Id,
    service_id: Id,
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
