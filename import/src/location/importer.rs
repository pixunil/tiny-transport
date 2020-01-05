use std::rc::Rc;
use std::error::Error;
use std::collections::HashMap;
use std::time::Instant;

use crate::utils::{Dataset, progress::elapsed};
use super::{Location, LocationId, LocationRecord, LocationImportError};

pub(crate) struct Importer;

impl Importer {
    pub(crate) fn import(dataset: &mut impl Dataset) -> Result<HashMap<LocationId, Rc<Location>>, Box<dyn Error>> {
        let mut queues = (Vec::new(), Vec::new());
        let mut locations = HashMap::new();

        let records = dataset.read_csv("stops.txt", "Importing locations")?;
        let started = Instant::now();
        for result in records {
            let record: LocationRecord = result?;
            record.import_or_enqueue(&mut locations, &mut queues)?;
        }

        for record in queues.0.into_iter().chain(queues.1) {
            if let Err(record) = record.try_import(&mut locations) {
                Err(LocationImportError::ParentNotFound(record))?;
            }
        }

        eprintln!("Imported {} locations in {:.2}s", locations.len(), elapsed(started));
        Ok(locations)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::{map, dataset};
    use crate::location::location::fixtures as locations;

    #[test]
    fn test_station_with_parent() {
        let mut dataset = dataset!(
            stops:
                stop_id,        stop_name,      stop_lat, stop_lon, location_type, parent_station;
                "hauptbahnhof", "Hauptbahnhof", 52.526,   13.369,   1,             "bahnhof"
        );

        let error = Importer::import(&mut dataset).unwrap_err();
        assert_eq!(format!("{}", error), "forbidden parent bahnhof for station hauptbahnhof");
    }

    #[test]
    fn test_child_missing_parent() {
        let mut dataset = dataset!(
            stops:
                stop_id,          stop_name,              stop_lat, stop_lon, location_type, parent_station;
                "hauptbahnhof_1", "Hauptbahnhof Gleis 1", 52.526,   13.369,   0,             "hauptbahnhof"
        );

        let error = Importer::import(&mut dataset).unwrap_err();
        assert_eq!(format!("{}", error), "parent hauptbahnhof for location hauptbahnhof_1 not found");
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = dataset!(
            stops:
                stop_id,         stop_name,       stop_lat, stop_lon, location_type, parent_station;
                "hauptbahnhof",  "Hauptbahnhof",  52.526,   13.369,   1,             "";
                "friedrichstr",  "Friedrichstr.", 52.520,   13.387,   1,             ""
        );

        let locations = Importer::import(&mut dataset).unwrap();
        assert_eq!(locations, map! {
            "hauptbahnhof" => Rc::new(locations::hauptbahnhof()),
            "friedrichstr" => Rc::new(locations::friedrichstr()),
        });
    }
}
