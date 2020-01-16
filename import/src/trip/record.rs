use std::rc::Rc;
use std::collections::HashMap;

use serde_derive::Deserialize;

use chrono::Duration;

use simulation::Direction;
use crate::deserialize;
use crate::service::{Service, ServiceId};
use crate::shape::ShapeId;
use crate::location::{Location, LocationId};
use crate::line::LineId;
use super::{TripBuffer, TripId};

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
    pub(super) fn import(self, id_mapping: &HashMap<LineId, usize>, services: &HashMap<ServiceId, Rc<Service>>, buffers: &mut HashMap<TripId, TripBuffer>) {
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
    pub(super) fn import(self, locations: &HashMap<LocationId, Rc<Location>>, buffers: &mut HashMap<TripId, TripBuffer>) {
        buffers.get_mut(&self.trip_id).unwrap()
            .add_stop(Rc::clone(&locations[&self.stop_id]), self.arrival_time, self.departure_time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{map, trip_buffer};
    use crate::service::fixtures::*;

    fn blue_trip_record() -> TripRecord {
        TripRecord {
            trip_id: "1".into(),
            route_id: "1".into(),
            service_id: "1".into(),
            shape_id: "1".into(),
            direction_id: Direction::Upstream,
        }
    }

    fn services() -> HashMap<ServiceId, Rc<Service>> {
        map! {
            "1" => Rc::new(services::mon_fri()),
        }
    }

    #[test]
    fn test_import_trip() {
        let record = blue_trip_record();
        let id_mapping = map! {
            "1" => 0,
        };
        let mut buffers = HashMap::new();
        record.import(&id_mapping, &services(), &mut buffers);
        assert_eq!(buffers, map! {
            "1" => trip_buffer!(blue, Upstream, 1, []),
        });
    }

    #[test]
    fn test_merges_lines() {
        let first = blue_trip_record();
        let mut second = blue_trip_record();
        second.trip_id = "2".into();
        second.route_id = "2".into();
        let id_mapping = map! {
            "1" => 0,
            "2" => 0,
        };
        let mut buffers = HashMap::new();
        first.import(&id_mapping, &services(), &mut buffers);
        second.import(&id_mapping, &services(), &mut buffers);
        assert_eq!(buffers, map! {
            "1" => trip_buffer!(blue, Upstream, 1, []),
            "2" => trip_buffer!(blue, Upstream, 1, []),
        });
    }
}
