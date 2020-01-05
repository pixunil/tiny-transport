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

    use crate::{map, shape, trip, route};
    use crate::location::fixtures::locations;

    #[macro_export]
    macro_rules! trip_buffer {
        ($line:expr, $service:ident, $shape:expr, $direction:ident, $start:expr, [$(($station:ident, $arrival:expr, $departure:expr)),* $(,)?]) => ({
            let service = Rc::new($crate::service::fixtures::services::$service());
            #[allow(unused_mut)]
            let mut trip_buffer = TripBuffer::new($line, service, $shape.into(), simulation::Direction::$direction);
            $(
                let location = Rc::new($crate::location::fixtures::locations::$station());
                trip_buffer.add_stop(location, Duration::minutes($start + $arrival), Duration::minutes($start + $departure));
            )*
            trip_buffer
        });
        (blue, Upstream, $start:expr, $locations:tt) => (
            $crate::trip_buffer!(0, mon_fri, "1", Upstream, $start, $locations)
        );
        (blue, Upstream, $start:expr) => (
            $crate::trip_buffer!(blue, Upstream, $start, [
                (hauptbahnhof, 0, 0),
                (friedrichstr, 4, 5),
                (hackescher_markt, 9, 9),
             ])
        );
        (blue, Downstream, $start:expr, $locations:tt) => (
            $crate::trip_buffer!(0, mon_fri, "2", Downstream, $start, $locations)
        );
        (blue, Downstream, $start:expr) => (
            $crate::trip_buffer!(blue, Downstream, $start, [
                (hackescher_markt, 0, 0),
                (friedrichstr, 4, 5),
                (hauptbahnhof, 9, 9),
             ])
        );
    }

    #[test]
    fn test_add_stop() {
        let mut buffer = trip_buffer!(blue, Upstream, 1, []);
        buffer.add_stop(Rc::new(locations::hauptbahnhof()), Duration::minutes(1), Duration::minutes(1));
        buffer.add_stop(Rc::new(locations::friedrichstr()), Duration::minutes(5), Duration::minutes(6));
        buffer.add_stop(Rc::new(locations::hackescher_markt()), Duration::minutes(10), Duration::minutes(10));
        assert_eq!(buffer.locations, [Rc::new(locations::hauptbahnhof()),
            Rc::new(locations::friedrichstr()), Rc::new(locations::hackescher_markt())]);
        assert_eq!(buffer.arrivals, vec![Duration::minutes(1), Duration::minutes(5), Duration::minutes(10)]);
        assert_eq!(buffer.departures, vec![Duration::minutes(1), Duration::minutes(6), Duration::minutes(10)]);
    }

    #[test]
    fn test_termini_for_upstream() {
        let buffer = trip_buffer!(blue, Upstream, 1);
        assert_eq!(buffer.termini(), (locations::hauptbahnhof().id, locations::hackescher_markt().id));
    }

    #[test]
    fn test_termini_for_downstream() {
        let buffer = trip_buffer!(blue, Downstream, 1);
        assert_eq!(buffer.termini(), (locations::hauptbahnhof().id, locations::hackescher_markt().id));
    }

    #[test]
    fn test_into_trip() {
        let buffer = trip_buffer!(blue, Upstream, 1);
        assert_eq!(buffer.into_trip(), trip!(blue, Upstream, 1));
        let buffer = trip_buffer!(blue, Upstream, 21);
        assert_eq!(buffer.into_trip(), trip!(blue, Upstream, 21));
    }

    fn shapes() -> HashMap<ShapeId, Shape> {
        map! {
            "1" => shape!(blue),
            "2" => shape!(blue reversed),
        }
    }

    #[test]
    fn test_create_route_with_upstream_buffer() {
        let mut routes = vec![HashMap::new()];
        let buffer = trip_buffer!(blue, Upstream, 1);
        buffer.place_into_routes(&shapes(), &mut routes);
        assert_eq!(routes[0], map! {
            (locations::hauptbahnhof().id, locations::hackescher_markt().id) => route!(blue, [(blue, Upstream, 1)]),
        });
    }

    #[test]
    fn test_create_route_with_downstream_buffer() {
        let mut routes = vec![HashMap::new()];
        let buffer = trip_buffer!(blue, Downstream, 1);
        buffer.place_into_routes(&shapes(), &mut routes);
        assert_eq!(routes[0], map! {
            (locations::hauptbahnhof().id, locations::hackescher_markt().id) => route!(blue, [(blue, Downstream, 1)]),
        });
    }

    #[test]
    fn test_add_trips_to_route() {
        let mut routes = vec![HashMap::new()];
        let buffer = trip_buffer!(blue, Upstream, 1);
        buffer.place_into_routes(&shapes(), &mut routes);
        let buffer = trip_buffer!(blue, Downstream, 1);
        buffer.place_into_routes(&shapes(), &mut routes);
        let buffer = trip_buffer!(blue, Upstream, 21);
        buffer.place_into_routes(&shapes(), &mut routes);
        assert_eq!(routes[0], map! {
            (locations::hauptbahnhof().id, locations::hackescher_markt().id) => route!(blue, [(blue, Upstream, 1), (blue, Downstream, 1), (blue, Upstream, 21)]),
        });
    }
}
