use std::rc::Rc;
use std::collections::HashMap;

use na::Point2;

use serde_derive::Deserialize;

use super::{Location, LocationId, LocationKind, LocationImportError};

#[derive(Debug, PartialEq, Deserialize)]
pub(super) struct LocationRecord {
    stop_id: LocationId,
    #[serde(rename = "location_type")]
    location_kind: LocationKind,
    parent_station: Option<LocationId>,
    stop_name: String,
    stop_lat: f32,
    stop_lon: f32,
}

impl LocationRecord {
    pub(super) fn stop_id(&self) -> &LocationId {
        &self.stop_id
    }

    pub(super) fn parent_station(&self) -> Option<&LocationId> {
        self.parent_station.as_ref()
    }

    pub(super) fn try_import(self, locations: &mut HashMap<LocationId, Rc<Location>>) -> Result<(), Self> {
        match self.parent_station {
            Some(ref parent_id) => {
                match locations.get(parent_id).cloned() {
                    Some(parent) => {
                        locations.insert(self.stop_id, parent);
                        Ok(())
                    },
                    None => {
                        Err(self)
                    },
                }
            },
            None => {
                let id = self.stop_id.clone();
                locations.insert(id, Rc::new(self.into()));
                Ok(())
            },
        }
    }

    pub(super) fn import_or_enqueue(self, locations: &mut HashMap<LocationId, Rc<Location>>, queues: &mut (Vec<Self>, Vec<Self>)) -> Result<(), LocationImportError> {
        if let Err(record) = self.try_import(locations) {
            match record.location_kind {
                LocationKind::Station => {
                    return Err(LocationImportError::StationHasParent(record));
                },
                LocationKind::Stop | LocationKind::Entrance | LocationKind::GenericNode => {
                    queues.0.push(record);
                },
                LocationKind::BoardingArea => {
                    queues.1.push(record);
                },
            }
        }
        Ok(())
    }
}

impl Into<Location> for LocationRecord {
    fn into(self) -> Location {
        let position = Point2::new(self.stop_lon, self.stop_lat);
        Location::new(self.stop_id, self.stop_name, position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::station;

    fn main_station_record() -> LocationRecord {
        LocationRecord {
            stop_id: "1".into(),
            location_kind: LocationKind::Station,
            parent_station: None,
            stop_name: "Main Station".to_string(),
            stop_lat: 52.526,
            stop_lon: 13.369,
        }
    }

    fn main_station_platform_record() -> LocationRecord {
        LocationRecord {
            stop_id: "2".into(),
            location_kind: LocationKind::Stop,
            parent_station: Some("1".into()),
            stop_name: "Main Station Platform 1".to_string(),
            stop_lat: 52.526,
            stop_lon: 13.369,
        }
    }

    #[test]
    fn test_into_location() {
        let location: Location = main_station_record().into();
        assert_eq!(location, station!(main_station));
    }

    #[test]
    fn test_import_parent() {
        let mut locations = HashMap::new();
        main_station_record().try_import(&mut locations).unwrap();
        assert_eq!(locations.len(), 1);
        assert_eq!(*locations[&"1".into()], station!(main_station));
    }

    #[test]
    fn test_import_child_without_parent() {
        let mut locations = HashMap::new();
        let record = main_station_platform_record().try_import(&mut locations).unwrap_err();
        assert_eq!(record, main_station_platform_record());
        assert!(locations.is_empty());
    }

    #[test]
    fn test_import_child_with_parent() {
        let mut locations = HashMap::new();
        locations.insert("1".into(), Rc::new(station!(main_station)));
        main_station_platform_record().try_import(&mut locations).unwrap();
        assert_eq!(locations.len(), 2);
        assert_eq!(*locations[&"2".into()], station!(main_station));
    }
}
