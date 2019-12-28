use std::error::Error;
use std::rc::Rc;
use std::iter;
use std::collections::HashMap;

use chrono::prelude::*;
use chrono::Duration;

use na::Point2;

use simulation::LineNode;

use crate::utils::*;
use crate::service::{Service, ServiceId};
use crate::shape::Shape;
use crate::location::{Location, LocationId};
use simulation::Direction;

#[derive(Debug, PartialEq)]
pub(crate) struct Route {
    pub(crate) locations: Vec<Rc<Location>>,
    shape: Shape,
    trips: Vec<Trip>,
}

impl Route {
    fn new(locations: Vec<Rc<Location>>, shape: Shape) -> Route {
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
                let y = -2000.0 * (waypoint.y - 105.04);
                LineNode::new(Point2::new(x, y))
            })
            .collect::<Vec<_>>();

        let mut lower = 0;
        for station in &self.locations[1 ..] {
            let (pos, node) = nodes.iter_mut()
                .enumerate()
                .skip(lower)
                .min_by(|(_, a), (_, b)| {
                    let a = na::distance(&station.position(), &a.position());
                    let b = na::distance(&station.position(), &b.position());
                    a.partial_cmp(&b).unwrap()
                })
                .unwrap();
            node.promote_to_stop();
            lower = pos;
        }

        let station = &self.locations[0];
        let second_stop = nodes.iter().position(LineNode::is_stop).unwrap();

        let node = nodes[.. second_stop].iter_mut()
            .min_by(|a, b| {
                let a = na::distance(&station.position(), &a.position());
                let b = na::distance(&station.position(), &b.position());
                a.partial_cmp(&b).unwrap()
            })
            .unwrap();
        node.promote_to_stop();

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
    durations: Vec<Duration>,
}

impl Trip {
    fn freeze(&self) -> serialization::Train {
        let durations = self.durations.iter()
            .map(|duration| duration.num_seconds() as u32)
            .collect();
        serialization::Train::new(self.direction, durations)
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

    fn add_stop(&mut self, record: StopRecord, locations: &HashMap<LocationId, Rc<Location>>) {
        let location = Rc::clone(&locations[&record.stop_id]);
        self.locations.push(location);
        self.arrivals.push(record.arrival_time);
        self.departures.push(record.departure_time);
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

        Trip {
            direction: self.direction,
            service: self.service,
            durations,
        }
    }

    fn place_into_routes(self, shapes: &HashMap<Id, Shape>, routes: &mut Vec<HashMap<(LocationId, LocationId), Route>>) {
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
        route.trips.push(self.into_trip());
    }
}

pub(crate) struct Importer<'a> {
    services: &'a HashMap<ServiceId, Rc<Service>>,
    locations: &'a HashMap<LocationId, Rc<Location>>,
    shapes: &'a HashMap<Id, Shape>,
    id_mapping: &'a HashMap<Id, usize>,
    num_lines: usize,
}

impl<'a> Importer<'a> {
    pub(crate) fn new(services: &'a HashMap<ServiceId, Rc<Service>>, locations: &'a HashMap<LocationId, Rc<Location>>,
        shapes: &'a HashMap<Id, Shape>, id_mapping: &'a HashMap<Id, usize>, num_lines: usize)
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

    pub(crate) fn import(self, dataset: &mut impl Dataset) -> Result<Vec<Vec<Route>>, Box<dyn Error>> {
        let mut buffers = self.import_trip_buffers(dataset)?;
        self.add_trip_stops(dataset, &mut buffers)?;

        let mut routes = iter::repeat_with(HashMap::new)
            .take(self.num_lines)
            .collect();
        for (_id, buffer) in buffers {
            buffer.place_into_routes(&self.shapes, &mut routes);
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
    service_id: ServiceId,
    shape_id: Id,
    #[serde(deserialize_with = "deserialize::direction")]
    direction_id: Direction,
}

#[derive(Debug, Deserialize)]
struct StopRecord {
    trip_id: Id,
    stop_id: LocationId,
    #[serde(deserialize_with = "deserialize::duration")]
    arrival_time: Duration,
    #[serde(deserialize_with = "deserialize::duration")]
    departure_time: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_abs_diff_eq;

    use crate::{service, station, shape};

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

    fn completed_trip_buffer() -> TripBuf {
        TripBuf {
            line_id: 0,
            service: Rc::new(service!(mon-fri)),
            shape_id: "1".to_string(),
            direction: Direction::Upstream,
            locations: station![main_station, center, market],
            arrivals: vec![Duration::minutes(1), Duration::minutes(5), Duration::minutes(10)],
            departures: vec![Duration::minutes(1), Duration::minutes(6), Duration::minutes(10)],
        }
    }

    #[test]
    fn test_add_stops_to_trip_buffer() {
        let records = vec![
            StopRecord {
                trip_id: "1".to_string(),
                stop_id: "1".into(),
                arrival_time: Duration::minutes(1),
                departure_time: Duration::minutes(1),
            },
            StopRecord {
                trip_id: "1".to_string(),
                stop_id: "2".into(),
                arrival_time: Duration::minutes(5),
                departure_time: Duration::minutes(6),
            },
            StopRecord {
                trip_id: "1".to_string(),
                stop_id: "3".into(),
                arrival_time: Duration::minutes(10),
                departure_time: Duration::minutes(10),
            },
        ];
        let mut locations = HashMap::new();
        locations.insert("1".into(), Rc::new(station!(main_station)));
        locations.insert("2".into(), Rc::new(station!(center)));
        locations.insert("3".into(), Rc::new(station!(market)));

        let mut buffer = empty_trip_buffer();
        for record in records {
            buffer.add_stop(record, &locations);
        }
        assert_eq!(buffer, completed_trip_buffer());
    }

    #[test]
    fn test_buffer_into_trip() {
        let buffer = completed_trip_buffer();
        let trip = Trip {
            direction: Direction::Upstream,
            service: Rc::new(service!(mon-fri)),
            durations: vec![Duration::minutes(1), Duration::minutes(0), Duration::minutes(4),
                Duration::minutes(1), Duration::minutes(4), Duration::minutes(0)],
        };
        assert_eq!(buffer.into_trip(), trip);
    }

    #[test]
    fn test_freeze_nodes_exact_shape() {
        let shape = shape!(52.526, 13.369; 52.523, 13.378; 52.520, 13.387; 52.521, 13.394; 52.523, 13.402);
        let route = Route::new(station![main_station, center, market], shape);
        let mut expected_nodes = [
            LineNode::new(Point2::new(-262.0, -24.0)),
            LineNode::new(Point2::new(-244.0, -12.0)),
            LineNode::new(Point2::new(-226.0,   0.0)),
            LineNode::new(Point2::new(-212.0,  -4.0)),
            LineNode::new(Point2::new(-196.0, -12.0)),
        ];
        expected_nodes[0].promote_to_stop();
        expected_nodes[2].promote_to_stop();
        expected_nodes[4].promote_to_stop();
        assert_abs_diff_eq!(*route.freeze_nodes(), expected_nodes, epsilon = 0.01);
    }

    #[test]
    fn test_freeze_nodes_circle() {
        let shape = shape!(52.549, 13.388; 52.503, 13.469; 52.475, 13.366; 52.501, 13.283; 52.549, 13.388);
        let route = Route::new(station![north_cross, east_cross, south_cross, west_cross, north_cross], shape);
        let mut expected_nodes = [
            LineNode::new(Point2::new(-224.0, -116.0)),
            LineNode::new(Point2::new( -62.0,   68.0)),
            LineNode::new(Point2::new(-268.0,  180.0)),
            LineNode::new(Point2::new(-434.0,   76.0)),
            LineNode::new(Point2::new(-224.0, -116.0)),
        ];
        expected_nodes[0].promote_to_stop();
        expected_nodes[1].promote_to_stop();
        expected_nodes[2].promote_to_stop();
        expected_nodes[3].promote_to_stop();
        expected_nodes[4].promote_to_stop();
        assert_abs_diff_eq!(*route.freeze_nodes(), expected_nodes, epsilon = 0.01);
    }
}
