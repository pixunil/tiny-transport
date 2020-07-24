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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset;
    use crate::fixtures::{locations, routes, services, shapes};
    use test_utils::{assert_eq_alternate, map};

    #[test]
    fn test_import_trip_buffers() {
        let mut dataset = dataset!(
            trips:
                trip_id, route_id, service_id, shape_id,                        direction_id;
                1,       tram_12,  mon_fri,    tram_12::oranienburger_tor_am_kupfergraben, 0;
                2,       tram_12,  mon_fri,    tram_12::am_kupfergraben_oranienburger_tor, 1
            stop_times:
                trip_id, stop_id,                    arrival_time, departure_time;
                1,       oranienburger_tor,          "9:02:00",    "9:02:00";
                1,       friedrichstr,               "9:04:00",    "9:04:00";
                1,       universitaetsstr,           "9:06:00",    "9:06:00";
                1,       am_kupfergraben,            "9:07:00",    "9:07:00";
                2,       am_kupfergraben,            "8:34:00",    "8:34:00";
                2,       georgenstr_am_kupfergraben, "8:35:00",    "8:35:00";
                2,       friedrichstr,               "8:38:00",    "8:38:00";
                2,       oranienburger_tor,          "8:40:00",    "8:40:00"
        );
        let id_mapping = map! {
            "tram_12" => 0,
        };

        let services = services::by_id();
        let locations = locations::by_id();
        let shapes = shapes::by_id();
        let importer = Importer::new(&services, &locations, &shapes, &id_mapping, 1);
        let routes = importer.import(&mut dataset).unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq_alternate!(
            routes[0],
            vec![routes::tram_12::oranienburger_tor_am_kupfergraben()],
        );
    }
}
