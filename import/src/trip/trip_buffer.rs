use std::collections::HashMap;
use std::rc::Rc;

use chrono::Duration;

use super::{RouteBuffer, Trip};
use crate::create_id_type;
use crate::location::{Location, LocationId};
use crate::service::Service;
use crate::shape::{Shape, ShapeId};
use simulation::Direction;

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
    pub(super) fn new(
        line_id: usize,
        service: Rc<Service>,
        shape_id: ShapeId,
        direction: Direction,
    ) -> TripBuffer {
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

    pub(super) fn add_stop(
        &mut self,
        location: Rc<Location>,
        arrival: Duration,
        departure: Duration,
    ) {
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

    pub(super) fn create_and_place_trip_by_terminus(
        self,
        shapes: &HashMap<ShapeId, Shape>,
        route_buffers: &mut Vec<HashMap<(LocationId, LocationId), RouteBuffer>>,
    ) {
        let route_buffer = route_buffers[self.line_id]
            .entry(self.termini())
            .or_insert_with(RouteBuffer::new);
        let durations = self.durations();
        let trip = Trip::new(self.direction, self.service, durations);
        route_buffer.add_trip(self.locations, &shapes[&self.shape_id], trip);
    }
}

#[cfg(test)]
pub(super) mod fixtures {
    macro_rules! trip_buffers {
        ($($line:ident: $line_id:expr, {$($trip:ident => $direction:ident, $service:ident, $shape:ident, [$($arrival:expr),*], [$($departure:expr),*]);* $(;)?}),* $(,)?) => (
            $(
                pub(in crate::trip) mod $line {
                    use simulation::Direction;
                    use crate::trip::fixtures::*;
                    use crate::trip::trip_buffer::*;

                    $(
                        pub(in crate::trip) fn $trip(hour: i64, minute: f64) -> TripBuffer {
                            #[allow(unused_variable)]
                            let start = hour * 3600 + (minute * 60.0) as i64;
                            TripBuffer {
                                line_id: $line_id,
                                service: Rc::new(services::$service()),
                                shape_id: stringify!($shape).into(),
                                direction: Direction::$direction,
                                locations: stop_locations::$line::$trip(),
                                arrivals: vec![$(
                                    Duration::seconds(start + ($arrival as f64 * 60.0) as i64)
                                ),*],
                                departures: vec![$(
                                    Duration::seconds(start + ($arrival as f64 * 60.0) as i64)
                                ),*],
                            }
                        }
                    )*
                }
            )*
        );
    }

    trip_buffers! {
        u4: 0, {
            empty => Upstream, mon_fri, u4, [], [];
            nollendorfplatz_innsbrucker_platz => Upstream, mon_fri, u4,
                [0, 2, 3.5, 5, 6], [0, 2, 3.5, 5, 6];
            innsbrucker_platz_nollendorfplatz => Downstream, mon_fri, u4,
                [0, 1, 2.5, 4, 6], [0, 1, 2.5, 4, 6];
        },
        tram_12: 0, {
            oranienburger_tor_am_kupfergraben => Upstream, mon_fri, oranienburger_tor_am_kupfergraben,
                [0, 2, 4, 5], [0, 2, 4, 5];
            am_kupfergraben_oranienburger_tor => Downstream, mon_fri, am_kupfergraben_oranienburger_tor,
                [0, 1, 4, 6], [0, 1, 4, 6];
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map;
    use crate::trip::fixtures::*;

    #[test]
    fn test_add_stop() {
        let mut buffer = trip_buffers::u4::empty(0, 0.0);
        buffer.add_stop(
            Rc::new(locations::innsbrucker_platz()),
            Duration::seconds(16560),
            Duration::seconds(16560),
        );
        assert_eq!(buffer.locations, [Rc::new(locations::innsbrucker_platz())]);
        assert_eq!(buffer.arrivals, vec![Duration::seconds(16560)]);
        assert_eq!(buffer.departures, vec![Duration::seconds(16560)]);
    }

    #[test]
    fn test_termini_for_upstream() {
        let buffer = trip_buffers::u4::nollendorfplatz_innsbrucker_platz(4, 36.0);
        assert_eq!(
            buffer.termini(),
            (
                locations::nollendorfplatz().id,
                locations::innsbrucker_platz().id
            )
        );
    }

    #[test]
    fn test_termini_for_downstream() {
        let buffer = trip_buffers::u4::innsbrucker_platz_nollendorfplatz(4, 46.0);
        assert_eq!(
            buffer.termini(),
            (
                locations::nollendorfplatz().id,
                locations::innsbrucker_platz().id
            )
        );
    }

    #[test]
    fn test_durations() {
        let buffer = trip_buffers::u4::nollendorfplatz_innsbrucker_platz(4, 36.0);
        let expected_durations = [16560, 0, 120, 0, 90, 0, 90, 0, 60, 0]
            .iter()
            .copied()
            .map(Duration::seconds)
            .collect::<Vec<_>>();
        assert_eq!(buffer.durations(), expected_durations);
        let buffer = trip_buffers::u4::nollendorfplatz_innsbrucker_platz(4, 46.0);
        let expected_durations = [17160, 0, 120, 0, 90, 0, 90, 0, 60, 0]
            .iter()
            .copied()
            .map(Duration::seconds)
            .collect::<Vec<_>>();
        assert_eq!(buffer.durations(), expected_durations);
    }

    #[test]
    fn test_create_route_with_upstream_buffer() {
        let mut route_buffers = vec![HashMap::new()];
        let buffer = trip_buffers::tram_12::oranienburger_tor_am_kupfergraben(9, 2.0);
        buffer.create_and_place_trip_by_terminus(&shapes::tram_12::by_id(), &mut route_buffers);
        assert_eq!(
            route_buffers[0],
            map! {
                (locations::oranienburger_tor().id, locations::am_kupfergraben().id)
                    => route_buffers::tram_12::with_1_upstream(),
            }
        );
    }

    #[test]
    fn test_create_route_with_downstream_buffer() {
        let mut route_buffers = vec![HashMap::new()];
        let buffer = trip_buffers::tram_12::am_kupfergraben_oranienburger_tor(8, 34.0);
        buffer.create_and_place_trip_by_terminus(&shapes::tram_12::by_id(), &mut route_buffers);
        assert_eq!(
            route_buffers[0],
            map! {
                (locations::oranienburger_tor().id, locations::am_kupfergraben().id)
                    => route_buffers::tram_12::with_1_downstream(),
            }
        );
    }

    #[test]
    fn test_add_trips_to_route() {
        let mut route_buffers = vec![HashMap::new()];
        let buffer = trip_buffers::tram_12::oranienburger_tor_am_kupfergraben(9, 2.0);
        buffer.create_and_place_trip_by_terminus(&shapes::tram_12::by_id(), &mut route_buffers);
        let buffer = trip_buffers::tram_12::am_kupfergraben_oranienburger_tor(8, 34.0);
        buffer.create_and_place_trip_by_terminus(&shapes::tram_12::by_id(), &mut route_buffers);
        assert_eq!(
            route_buffers[0],
            map! {
                (locations::oranienburger_tor().id, locations::am_kupfergraben().id)
                    => route_buffers::tram_12::with_1_upstream_1_downstream(),
            }
        );
    }
}
