use std::rc::Rc;
use std::collections::HashMap;

use serde_derive::Deserialize;

use chrono::Duration;

use simulation::Direction;
use crate::utils::deserialize;
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
