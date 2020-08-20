use std::collections::HashMap;
use std::rc::Rc;

use serde_derive::Deserialize;

use chrono::Duration;

use super::{TripBuffer, TripId};
use crate::deserialize;
use crate::line::LineId;
use crate::location::{Location, LocationId};
use crate::service::{Service, ServiceId};
use crate::shape::ShapeId;
use simulation::Direction;

#[derive(Debug, Deserialize)]
pub(super) struct TripRecord {
    trip_id: TripId,
    route_id: LineId,
    service_id: ServiceId,
    shape_id: ShapeId,
    #[serde(deserialize_with = "deserialize::direction")]
    direction_id: Direction,
}

impl TripRecord {
    pub(super) fn import(
        self,
        id_mapping: &HashMap<LineId, usize>,
        services: &HashMap<ServiceId, Rc<Service>>,
        buffers: &mut HashMap<TripId, TripBuffer>,
    ) {
        let line_id = id_mapping[&self.route_id];
        let service = Rc::clone(&services[&self.service_id]);
        let buffer = TripBuffer::new(line_id, service, self.shape_id, self.direction_id);
        buffers.insert(self.trip_id, buffer);
    }
}

#[derive(Debug, Deserialize)]
pub(super) struct StopRecord {
    trip_id: TripId,
    stop_id: LocationId,
    #[serde(deserialize_with = "deserialize::duration")]
    arrival_time: Duration,
    #[serde(deserialize_with = "deserialize::duration")]
    departure_time: Duration,
}

impl StopRecord {
    pub(super) fn import(
        self,
        locations: &HashMap<LocationId, Rc<Location>>,
        buffers: &mut HashMap<TripId, TripBuffer>,
    ) {
        buffers.get_mut(&self.trip_id).unwrap().add_stop(
            Rc::clone(&locations[&self.stop_id]),
            self.arrival_time,
            self.departure_time,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{locations, services, trip_buffers};
    use test_utils::{assert_eq_alternate, map, time};

    fn u4_trip_record() -> TripRecord {
        TripRecord {
            trip_id: "u4_0".into(),
            route_id: "u4".into(),
            service_id: "mon_fri".into(),
            shape_id: "u4::u4".into(),
            direction_id: Direction::Upstream,
        }
    }

    #[test]
    fn test_import_trip() {
        let record = u4_trip_record();
        let id_mapping = map! {
            "u4" => 0,
        };
        let mut buffers = HashMap::new();
        record.import(&id_mapping, &services::by_id(), &mut buffers);
        assert_eq_alternate!(
            buffers,
            map! {
                "u4_0" => trip_buffers::u4::empty(time!(0:00)),
            }
        );
    }

    #[test]
    fn test_merges_lines() {
        let first = u4_trip_record();
        let mut second = u4_trip_record();
        second.trip_id = "u4_duplicate_0".into();
        second.route_id = "u4_duplicate".into();
        let id_mapping = map! {
            "u4" => 0,
            "u4_duplicate" => 0,
        };
        let mut buffers = HashMap::new();
        first.import(&id_mapping, &services::by_id(), &mut buffers);
        second.import(&id_mapping, &services::by_id(), &mut buffers);
        assert_eq_alternate!(
            buffers,
            map! {
                "u4_0" => trip_buffers::u4::empty(time!(0:00)),
                "u4_duplicate_0" => trip_buffers::u4::empty(time!(0:00)),
            }
        );
    }

    #[test]
    fn test_import_stops() {
        let records = vec![
            StopRecord {
                trip_id: "u4_0".into(),
                stop_id: "nollendorfplatz".into(),
                arrival_time: Duration::seconds(time!(4:36:00)),
                departure_time: Duration::seconds(time!(4:36:00)),
            },
            StopRecord {
                trip_id: "u4_0".into(),
                stop_id: "viktoria_luise_platz".into(),
                arrival_time: Duration::seconds(time!(4:38:00)),
                departure_time: Duration::seconds(time!(4:38:00)),
            },
            StopRecord {
                trip_id: "u4_0".into(),
                stop_id: "bayerischer_platz".into(),
                arrival_time: Duration::seconds(time!(4:39:30)),
                departure_time: Duration::seconds(time!(4:39:30)),
            },
            StopRecord {
                trip_id: "u4_0".into(),
                stop_id: "rathaus_schoeneberg".into(),
                arrival_time: Duration::seconds(time!(4:41:00)),
                departure_time: Duration::seconds(time!(4:41:00)),
            },
            StopRecord {
                trip_id: "u4_0".into(),
                stop_id: "innsbrucker_platz".into(),
                arrival_time: Duration::seconds(time!(4:42:00)),
                departure_time: Duration::seconds(time!(4:42:00)),
            },
        ];

        let mut buffers = map! {
            "u4_0" => trip_buffers::u4::empty(time!(0:00)),
        };

        for record in records {
            record.import(&locations::by_id(), &mut buffers);
        }

        assert_eq_alternate!(
            buffers,
            map! {
                "u4_0" => trip_buffers::u4::nollendorfplatz_innsbrucker_platz(time!(4:36:00)),
            }
        );
    }
}
