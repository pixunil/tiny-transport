use std::rc::Rc;
use std::collections::HashMap;

use chrono::Duration;

use simulation::Direction;
use crate::create_id_type;
use crate::service::Service;
use crate::shape::{Shape, ShapeId};
use crate::location::{Location, LocationId};
use super::{Trip, RouteBuffer};

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

    fn durations(&self) -> Vec<Duration> {
        let mut durations = Vec::new();
        for (i, &arrival) in self.arrivals.iter().enumerate() {
            if i == 0 {
                durations.push(arrival);
            } else {
                durations.push(arrival - self.departures[i - 1]);
            }
            durations.push(self.departures[i] - arrival);
        }
        durations
    }

    pub(super) fn create_and_place_trip_by_terminus(self, shapes: &HashMap<ShapeId, Shape>, route_buffers: &mut Vec<HashMap<(LocationId, LocationId), RouteBuffer>>) {
        let route_buffer = route_buffers[self.line_id].entry(self.termini())
            .or_insert_with(RouteBuffer::new);
        let durations = self.durations();
        let trip = Trip::new(self.direction, self.service, durations);
        route_buffer.add_trip(self.locations, &shapes[&self.shape_id], trip);
    }
}

#[cfg(test)]
pub(super) mod fixtures {
    macro_rules! trip_buffers {
        ($($line:ident: $line_id:expr, {$($trip:ident => $direction:ident, $service:ident, [$($arrival:expr),*], [$($departure:expr),*]);* $(;)?}),* $(,)?) => (
            $(
                pub(in crate::trip) mod $line {
                    use simulation::Direction;
                    use crate::trip::fixtures::*;
                    use crate::trip::trip_buffer::*;

                    $(
                        pub(in crate::trip) fn $trip(start: i64) -> TripBuffer {
                            TripBuffer {
                                line_id: $line_id,
                                service: Rc::new(services::$service()),
                                shape_id: stringify!($trip).into(),
                                direction: Direction::$direction,
                                locations: stop_locations::$line::$trip(),
                                arrivals: vec![$(Duration::minutes(start + $arrival)),*],
                                departures: vec![$(Duration::minutes(start + $departure)),*],
                            }
                        }
                    )*
                }
            )*
        );
    }

    trip_buffers! {
        tram_12: 0, {
            oranienburger_tor_am_kupfergraben => Upstream, mon_fri, [0, 2, 4, 5], [0, 2, 4, 5];
            am_kupfergraben_oranienburger_tor => Downstream, mon_fri, [0, 1, 4, 6], [0, 1, 4, 6];
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::map;
    use crate::trip::fixtures::*;
    use super::fixtures as trip_buffers;

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
    fn test_durations() {
        let buffer = trip_buffer!(blue, Upstream, 1);
        let expected_durations = [1, 0, 4, 1, 4, 0].iter().copied().map(Duration::minutes).collect::<Vec<_>>();
        assert_eq!(buffer.durations(), expected_durations);
        let buffer = trip_buffer!(blue, Upstream, 21);
        let expected_durations = [21, 0, 4, 1, 4, 0].iter().copied().map(Duration::minutes).collect::<Vec<_>>();
        assert_eq!(buffer.durations(), expected_durations);
    }

    #[test]
    fn test_create_route_with_upstream_buffer() {
        let mut route_buffers = vec![HashMap::new()];
        let buffer = trip_buffers::tram_12::oranienburger_tor_am_kupfergraben(542);
        buffer.create_and_place_trip_by_terminus(&shapes::tram_12::by_id(), &mut route_buffers);
        assert_eq!(route_buffers[0], map! {
            (locations::oranienburger_tor().id, locations::am_kupfergraben().id) => route_buffers::tram_12::with_1_upstream(),
        });
    }

    #[test]
    fn test_create_route_with_downstream_buffer() {
        let mut route_buffers = vec![HashMap::new()];
        let buffer = trip_buffers::tram_12::am_kupfergraben_oranienburger_tor(514);
        buffer.create_and_place_trip_by_terminus(&shapes::tram_12::by_id(), &mut route_buffers);
        assert_eq!(route_buffers[0], map! {
            (locations::oranienburger_tor().id, locations::am_kupfergraben().id) => route_buffers::tram_12::with_1_downstream(),
        });
    }

    #[test]
    fn test_add_trips_to_route() {
        let mut route_buffers = vec![HashMap::new()];
        let buffer = trip_buffers::tram_12::oranienburger_tor_am_kupfergraben(542);
        buffer.create_and_place_trip_by_terminus(&shapes::tram_12::by_id(), &mut route_buffers);
        let buffer = trip_buffers::tram_12::am_kupfergraben_oranienburger_tor(514);
        buffer.create_and_place_trip_by_terminus(&shapes::tram_12::by_id(), &mut route_buffers);
        assert_eq!(route_buffers[0], map! {
            (locations::oranienburger_tor().id, locations::am_kupfergraben().id) => route_buffers::tram_12::with_1_upstream_1_downstream(),
        });
    }
}
