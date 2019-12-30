use std::rc::Rc;
use std::error::Error;
use std::collections::HashMap;

use crate::utils::Dataset;
use super::{Location, LocationId, LocationRecord, LocationImportError};

pub(crate) struct Importer;

impl Importer {
    pub(crate) fn import(dataset: &mut impl Dataset) -> Result<HashMap<LocationId, Rc<Location>>, Box<dyn Error>> {
        let mut queues = (Vec::new(), Vec::new());
        let mut locations = HashMap::new();
        let mut reader = dataset.read_csv("stops.txt")?;
        for result in reader.deserialize() {
            let record: LocationRecord = result?;
            record.import_or_enqueue(&mut locations, &mut queues)?;
        }

        for record in queues.0.into_iter().chain(queues.1) {
            if let Err(record) = record.try_import(&mut locations) {
                Err(LocationImportError::ParentNotFound(record))?;
            }
        }

        Ok(locations)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::{map, dataset, station};

    #[test]
    fn test_station_with_parent() {
        let mut dataset = dataset!(
            stops:
                stop_id, stop_name,      stop_lat, stop_lon, location_type, parent_station;
                1,       "Main Station", 52.526,   13.369,   1,             10
        );

        let error = Importer::import(&mut dataset).unwrap_err();
        assert_eq!(format!("{}", error), "forbidden parent 10 for station 1");
    }

    #[test]
    fn test_child_missing_parent() {
        let mut dataset = dataset!(
            stops:
                stop_id, stop_name,                 stop_lat, stop_lon, location_type, parent_station;
                2,       "Main Station Platform 1", 52.526,   13.369,   0,             1
        );

        let error = Importer::import(&mut dataset).unwrap_err();
        assert_eq!(format!("{}", error), "parent 1 for location 2 not found");
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = dataset!(
            stops:
                stop_id, stop_name,      stop_lat, stop_lon, location_type, parent_station;
                1,       "Main Station", 52.526,   13.369,   1,             "";
                2,       "Center",       52.520,   13.387,   1,             ""
        );

        let locations = Importer::import(&mut dataset).unwrap();
        assert_eq!(locations, map! {
            "1" => Rc::new(station!(main_station)),
            "2" => Rc::new(station!(center)),
        });
    }
}
