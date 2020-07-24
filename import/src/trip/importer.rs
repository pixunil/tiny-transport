use std::collections::HashMap;
use std::error::Error;
use std::iter;
use std::rc::Rc;

use super::{Route, RouteBuffer, StopRecord, TripBuffer, TripId, TripRecord};
use crate::line::LineId;
use crate::location::{Location, LocationId};
use crate::service::{Service, ServiceId};
use crate::shape::{Shape, ShapeId};
use crate::utils::{Action, Dataset};

pub(crate) struct Importer<'a> {
    services: &'a HashMap<ServiceId, Rc<Service>>,
    locations: &'a HashMap<LocationId, Rc<Location>>,
    shapes: &'a HashMap<ShapeId, Shape>,
    id_mapping: &'a HashMap<LineId, usize>,
    line_count: usize,
}

impl<'a> Importer<'a> {
    pub(crate) fn new(
        services: &'a HashMap<ServiceId, Rc<Service>>,
        locations: &'a HashMap<LocationId, Rc<Location>>,
        shapes: &'a HashMap<ShapeId, Shape>,
        id_mapping: &'a HashMap<LineId, usize>,
        line_count: usize,
    ) -> Importer<'a> {
        Importer {
            services,
            locations,
            shapes,
            id_mapping,
            line_count,
        }
    }

    fn import_trip_buffers(
        &self,
        dataset: &mut impl Dataset,
    ) -> Result<HashMap<TripId, TripBuffer>, Box<dyn Error>> {
        let mut buffers = HashMap::new();

        let action = Action::start("Importing trips");
        for result in action.read_csv(dataset, "trips.txt")? {
            let record: TripRecord = result?;
            record.import(self.id_mapping, self.services, &mut buffers);
        }
        action.complete(&format!("Imported {} trips", buffers.len()));
        Ok(buffers)
    }

    fn add_trip_stops(
        &self,
        dataset: &mut impl Dataset,
        buffers: &mut HashMap<TripId, TripBuffer>,
    ) -> Result<(), Box<dyn Error>> {
        let action = Action::start("Importing trip stops");
        for result in action.read_csv(dataset, "stop_times.txt")? {
            let record: StopRecord = result?;
            record.import(self.locations, buffers);
        }
        action.complete("Imported trip stops");
        Ok(())
    }

    fn combine_into_routes(&self, buffers: HashMap<TripId, TripBuffer>) -> Vec<Vec<Route>> {
        let mut action = Action::start("Assigning trips to their lines");
        let mut route_buffers = iter::repeat_with(RouteBuffer::new)
            .take(self.line_count)
            .collect();

        for (_, buffer) in action.wrap_iter(buffers) {
            buffer.create_and_place_trip(&self.shapes, &mut route_buffers);
        }
        action.complete("Assigned trips to their lines");

        let mut action = Action::start("Merging trips into routes");
        let routes = action
            .wrap_iter(route_buffers)
            .map(|route_buffer| route_buffer.into_routes())
            .collect();
        action.complete("Merged trips into routes");
        routes
    }

    pub(crate) fn import(
        self,
        dataset: &mut impl Dataset,
    ) -> Result<Vec<Vec<Route>>, Box<dyn Error>> {
        let mut buffers = self.import_trip_buffers(dataset)?;
        self.add_trip_stops(dataset, &mut buffers)?;
        Ok(self.combine_into_routes(buffers))
    }
}
