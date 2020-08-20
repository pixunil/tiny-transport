use std::rc::Rc;

use chrono::Duration;

use super::{RouteBuffer, Trip};
use crate::create_id_type;
use crate::location::Location;
use crate::service::Service;
use crate::shape::{ShapeId, Shapes};
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

    pub(super) fn create_and_place_trip<'a>(
        self,
        shapes: &'a Shapes,
        route_buffers: &mut Vec<RouteBuffer<'a>>,
    ) {
        let durations = self.durations();
        let trip = Trip::new(self.direction, self.service, durations);
        let route_buffer = &mut route_buffers[self.line_id];
        route_buffer.add_trip(self.locations, shapes.bind(&self.shape_id), trip);
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    macro_rules! trip_buffers {
        ($(
            $line:ident: $line_id:expr, {
                $(
                    $trip:ident => $direction:ident, $service:ident, $shape:ident,
                    $arrival_times:tt, $departure_times:tt
                );* $(;)?
            }
        ),* $(,)?) => (
            $(
                pub(in crate::trip) mod $line {
                    use simulation::Direction;
                    use crate::fixtures::{services, stop_locations};
                    use crate::trip::trip_buffer::*;
                    use test_utils::times;

                    $(
                        pub(in crate::trip) fn $trip(start: i64) -> TripBuffer {
                            TripBuffer {
                                line_id: $line_id,
                                service: Rc::new(services::$service()),
                                shape_id: format!("{}::{}", stringify!($line), stringify!($shape)).as_str().into(),
                                direction: Direction::$direction,
                                locations: stop_locations::$line::$trip(),
                                arrivals: times!(Duration; +start, $arrival_times),
                                departures: times!(Duration; +start, $departure_times),
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
                [0:00, 2:00, 3:30, 5:00, 6:00], [0:00, 2:00, 3:30, 5:00, 6:00];
        },
        tram_12: 0, {
            oranienburger_tor_am_kupfergraben => Upstream, mon_fri, oranienburger_tor_am_kupfergraben,
                [0:00, 2:00, 4:00, 5:00], [0:00, 2:00, 4:00, 5:00];
            am_kupfergraben_oranienburger_tor => Downstream, mon_fri, am_kupfergraben_oranienburger_tor,
                [0:00, 1:00, 4:00, 6:00], [0:00, 1:00, 4:00, 6:00];
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{locations, route_buffers, shapes, trip_buffers};
    use test_utils::{assert_eq_alternate, time, times};

    #[test]
    fn test_add_stop() {
        let mut buffer = trip_buffers::u4::empty(time!(0:00));
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
    fn test_durations() {
        let buffer = trip_buffers::u4::nollendorfplatz_innsbrucker_platz(time!(4:36:00));
        let expected_durations = times!(Duration; 4:36:00,
            0:00, 2:00, 0:00, 1:30, 0:00, 1:30, 0:00, 1:00, 0:00);
        assert_eq!(buffer.durations(), expected_durations);
        let buffer = trip_buffers::u4::nollendorfplatz_innsbrucker_platz(time!(4:46:00));
        let expected_durations = times!(Duration; 4:46:00,
            0:00, 2:00, 0:00, 1:30, 0:00, 1:30, 0:00, 1:00, 0:00);
        assert_eq!(buffer.durations(), expected_durations);
    }

    #[test]
    fn test_create_route_with_upstream_buffer() {
        let shapes = shapes::tram_12::by_id();
        let mut route_buffers = vec![RouteBuffer::new()];
        let buffer = trip_buffers::tram_12::oranienburger_tor_am_kupfergraben(time!(9:02:00));
        buffer.create_and_place_trip(&shapes, &mut route_buffers);
        assert_eq_alternate!(
            route_buffers[0],
            route_buffers::tram_12::with_1_upstream(&shapes)
        );
    }

    #[test]
    fn test_create_route_with_downstream_buffer() {
        let shapes = shapes::tram_12::by_id();
        let mut route_buffers = vec![RouteBuffer::new()];
        let buffer = trip_buffers::tram_12::am_kupfergraben_oranienburger_tor(time!(8:34:00));
        buffer.create_and_place_trip(&shapes, &mut route_buffers);
        assert_eq_alternate!(
            route_buffers[0],
            route_buffers::tram_12::with_1_downstream(&shapes)
        );
    }

    #[test]
    fn test_add_trips_to_route() {
        let shapes = shapes::tram_12::by_id();
        let mut route_buffers = vec![RouteBuffer::new()];
        let buffer = trip_buffers::tram_12::oranienburger_tor_am_kupfergraben(time!(9:02:00));
        buffer.create_and_place_trip(&shapes, &mut route_buffers);
        let buffer = trip_buffers::tram_12::am_kupfergraben_oranienburger_tor(time!(8:34:00));
        buffer.create_and_place_trip(&shapes, &mut route_buffers);
        assert_eq_alternate!(
            route_buffers[0],
            route_buffers::tram_12::with_1_upstream_1_downstream(&shapes)
        );
    }
}
