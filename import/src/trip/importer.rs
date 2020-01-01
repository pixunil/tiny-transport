use std::error::Error;
use std::rc::Rc;
use std::iter;
use std::collections::HashMap;
use std::time::Instant;

use crate::utils::{Dataset, progress::elapsed};
use crate::service::{Service, ServiceId};
use crate::shape::{Shape, ShapeId};
use crate::location::{Location, LocationId};
use crate::line::LineId;
use super::{TripBuffer, TripId, Route, TripRecord, StopRecord};

pub(crate) struct Importer<'a> {
    services: &'a HashMap<ServiceId, Rc<Service>>,
    locations: &'a HashMap<LocationId, Rc<Location>>,
    shapes: &'a HashMap<ShapeId, Shape>,
    id_mapping: &'a HashMap<LineId, usize>,
    num_lines: usize,
}

impl<'a> Importer<'a> {
    pub(crate) fn new(services: &'a HashMap<ServiceId, Rc<Service>>, locations: &'a HashMap<LocationId, Rc<Location>>,
                      shapes: &'a HashMap<ShapeId, Shape>, id_mapping: &'a HashMap<LineId, usize>, num_lines: usize)
                      -> Importer<'a>
    {
        Importer { services, locations, shapes, id_mapping, num_lines }
    }

    fn import_trip_buffers(&self, dataset: &mut impl Dataset) -> Result<HashMap<TripId, TripBuffer>, Box<dyn Error>> {
        let mut buffers = HashMap::new();

        let records = dataset.read_csv("trips.txt", "Importing trips")?;
        let started = Instant::now();
        for result in records {
            let record: TripRecord = result?;
            record.import(self.id_mapping, self.services, &mut buffers);
        }

        eprintln!("Imported {} trips in {:.2}s", buffers.len(), elapsed(started));
        Ok(buffers)
    }

    fn add_trip_stops(&self, dataset: &mut impl Dataset, buffers: &mut HashMap<TripId, TripBuffer>) -> Result<(), Box<dyn Error>> {
        let records = dataset.read_csv("stop_times.txt", "Importing trip stops")?;
        let started = Instant::now();
        for result in records {
            let record: StopRecord = result?;
            record.import(self.locations, buffers);
        }

        eprintln!("Imported trip stops in {:.2}s", elapsed(started));
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
