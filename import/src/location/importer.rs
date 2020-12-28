use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;

use super::{Location, LocationId, LocationImportError, LocationRecord};
use crate::utils::{Action, Dataset};

pub(crate) struct Importer;

impl Importer {
    pub(crate) fn import(
        dataset: &mut impl Dataset,
    ) -> Result<HashMap<LocationId, Rc<Location>>, Box<dyn Error>> {
        let mut queues = (Vec::new(), Vec::new());
        let mut locations = HashMap::new();

        let action = Action::start("Importing locations");
        for result in action.read_csv(dataset, "stops.txt")? {
            let record: LocationRecord = result?;
            record.import_or_enqueue(&mut locations, &mut queues)?;
        }

        for record in queues.0.into_iter().chain(queues.1) {
            if let Err(record) = record.try_import(&mut locations) {
                return Err(LocationImportError::ParentNotFound(record).into());
            }
        }

        action.complete(&format!("Imported {} locations", locations.len()));
        Ok(locations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset;
    use crate::fixtures::locations;
    use common::map;

    #[test]
    fn test_station_with_parent() {
        let mut dataset = dataset!(
            stops:
                stop_id,        stop_name,      stop_lat, stop_lon, location_type, parent_station;
                "hauptbahnhof", "Hauptbahnhof", 52.526,   13.369,   1,             "bahnhof"
        );

        assert_eq!(
            Importer::import(&mut dataset).unwrap_err().to_string(),
            "forbidden parent bahnhof for station hauptbahnhof"
        );
    }

    #[test]
    fn test_child_missing_parent() {
        let mut dataset = dataset!(
            stops:
                stop_id,          stop_name,              stop_lat, stop_lon, location_type, parent_station;
                "hauptbahnhof_1", "Hauptbahnhof Gleis 1", 52.526,   13.369,   0,             "hauptbahnhof"
        );

        assert_eq!(
            Importer::import(&mut dataset).unwrap_err().to_string(),
            "parent hauptbahnhof for location hauptbahnhof_1 not found"
        );
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = dataset!(
            stops:
                stop_id,         stop_name,       stop_lat, stop_lon, location_type, parent_station;
                "hauptbahnhof",  "Hauptbahnhof",  52.526,   13.369,   1,             "";
                "friedrichstr",  "Friedrichstr.", 52.520,   13.387,   1,             ""
        );

        assert_eq!(
            Importer::import(&mut dataset).unwrap(),
            map! {
                "hauptbahnhof" => Rc::new(locations::hauptbahnhof()),
                "friedrichstr" => Rc::new(locations::friedrichstr()),
            }
        );
    }
}
