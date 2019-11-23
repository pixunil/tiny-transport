use std::error::Error;
use std::rc::Rc;
use std::fmt;
use std::cmp::Ordering;
use std::collections::HashMap;

use na::Point2;

use serde::Deserializer;
use serde::de::{Deserialize, Visitor, Error as DeserializeError};

use super::utils::*;

#[derive(Debug, PartialEq)]
pub struct Location {
    pub id: Id,
    pub name: String,
    position: Point2<f32>,
}

impl Location {
    pub fn new(id: Id, name: String, position: Point2<f32>) -> Location {
        Location { id, name, position }
    }

    pub fn position(&self) -> Point2<f32> {
        let x = 2000.0 * (self.position.x - 13.5);
        let y = -4000.0 * (self.position.y - 52.52);
        Point2::new(x, y)
    }

    pub fn station_cmp(&self, other: &Location) -> Ordering {
        self.id.cmp(&other.id)
    }

    pub fn freeze(&self) -> serialization::Station {
        serialization::Station::new(self.position(), self.name.clone())
    }
}

impl From<LocationRecord> for Location {
    fn from(record: LocationRecord) -> Location {
        let position = Point2::new(record.stop_lon, record.stop_lat);
        Location::new(record.stop_id, record.stop_name, position)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LocationType {
    Stop,
    Station,
    Entrance,
    GenericNode,
    BoardingArea,
}

impl<'de> Deserialize<'de> for LocationType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct LineKindVisitor;

        impl<'de> Visitor<'de> for LineKindVisitor {
            type Value = LocationType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("integer")
            }

            fn visit_u64<E>(self, value: u64) -> Result<LocationType, E>
                where E: DeserializeError
            {
                match value {
                    0 => Ok(LocationType::Stop),
                    1 => Ok(LocationType::Station),
                    2 => Ok(LocationType::Entrance),
                    3 => Ok(LocationType::GenericNode),
                    4 => Ok(LocationType::BoardingArea),
                    _ => Err(E::custom(format!("unknown location type of value: {}", value))),
                }
            }
        }

        deserializer.deserialize_u64(LineKindVisitor)
    }
}

#[derive(Debug)]
enum LocationImportError {
    StationHasParent(LocationRecord),
    ParentNotFound(LocationRecord),
}

impl fmt::Display for LocationImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocationImportError::StationHasParent(record) => {
                write!(formatter, "forbidden parent {} for station {}", record.parent_station.as_ref().unwrap(), record.stop_id)
            },
            LocationImportError::ParentNotFound(record) => {
                write!(formatter, "parent {} for location {} not found", record.parent_station.as_ref().unwrap(), record.stop_id)
            }
        }
    }
}

impl Error for LocationImportError {}

pub struct Importer;

impl Importer {
    pub fn import(dataset: &mut impl Dataset) -> Result<HashMap<Id, Rc<Location>>, Box<dyn Error>> {
        let mut queues = (Vec::new(), Vec::new());
        let mut locations = HashMap::new();
        let mut reader = dataset.read_csv("stops.txt")?;
        for result in reader.deserialize() {
            if let Err(record) = Self::process_record(result?, &mut locations) {
                match record.location_type {
                    LocationType::Station => {
                        Err(LocationImportError::StationHasParent(record))?;
                    },
                    LocationType::Stop | LocationType::Entrance | LocationType::GenericNode => {
                        queues.0.push(record);
                    },
                    LocationType::BoardingArea => {
                        queues.1.push(record);
                    },
                }
            }
        }

        for record in queues.0.into_iter().chain(queues.1) {
            if let Err(record) = Self::process_record(record, &mut locations) {
                Err(LocationImportError::ParentNotFound(record))?;
            }
        }

        Ok(locations)
    }

    fn process_record(record: LocationRecord, locations: &mut HashMap<Id, Rc<Location>>) -> Result<(), LocationRecord> {
        match record.parent_station {
            Some(ref parent_id) => {
                match locations.get(parent_id).cloned() {
                    Some(parent) => {
                        locations.insert(record.stop_id, parent);
                        Ok(())
                    },
                    None => {
                        Err(record)
                    },
                }
            },
            None => {
                let id = record.stop_id.clone();
                locations.insert(id, Rc::new(Location::from(record)));
                Ok(())
            },
        }
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct LocationRecord {
    stop_id: Id,
    location_type: LocationType,
    parent_station: Option<Id>,
    stop_name: String,
    stop_lat: f32,
    stop_lon: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_test::{Token, assert_de_tokens, assert_de_tokens_error};

    #[macro_export]
    macro_rules! station {
        ($id:expr, $name:expr, $lat:expr, $lon:expr) => (
            $crate::location::Location::new($id.to_string(), $name.to_string(), Point2::new($lon, $lat))
        );
        (main_station) => (
            $crate::station!("1", "Main Station", 52.526, 13.369)
        );
        (center) => (
            $crate::station!("2", "Center", 52.520, 13.387)
        );
        (market) => (
            $crate::station!("3", "Market", 52.523, 13.402)
        );
        (north_cross) => (
            $crate::station!("4", "North Cross", 52.549, 13.388)
        );
        (east_cross) => (
            $crate::station!("5", "East Cross", 52.503, 13.469)
        );
        (south_cross) => (
            $crate::station!("6", "South Cross", 52.475, 13.366)
        );
        (west_cross) => (
            $crate::station!("7", "West Cross", 52.501, 13.283)
        );
        ($($station:ident),*) => (
            vec![$(Rc::new($crate::station!($station))),*]
        );
    }

    fn main_station_record() -> LocationRecord {
        LocationRecord {
            stop_id: "1".to_string(),
            location_type: LocationType::Station,
            parent_station: None,
            stop_name: "Main Station".to_string(),
            stop_lat: 52.526,
            stop_lon: 13.369,
        }
    }

    fn main_station_platform_record() -> LocationRecord {
        LocationRecord {
            stop_id: "2".to_string(),
            location_type: LocationType::Stop,
            parent_station: Some("1".to_string()),
            stop_name: "Main Station Platform 1".to_string(),
            stop_lat: 52.526,
            stop_lon: 13.369,
        }
    }

    #[test]
    fn test_deserialize_location_type() {
        assert_de_tokens(&LocationType::Stop, &[Token::U16(0)]);
        assert_de_tokens(&LocationType::Station, &[Token::U16(1)]);
        assert_de_tokens(&LocationType::Entrance, &[Token::U16(2)]);
        assert_de_tokens(&LocationType::GenericNode, &[Token::U16(3)]);
        assert_de_tokens(&LocationType::BoardingArea, &[Token::U16(4)]);
        assert_de_tokens_error::<LocationType>(&[Token::U16(5)],
                                           "unknown location type of value: 5");
        assert_de_tokens_error::<LocationType>(&[Token::Str("")],
                                           "invalid type: string \"\", expected integer");
    }

    #[test]
    fn test_import_location() {
        let location = Location::from(main_station_record());
        assert_eq!(location, station!(main_station));
    }

    #[test]
    fn test_process_parent() {
        let mut locations = HashMap::new();
        Importer::process_record(main_station_record(), &mut locations).unwrap();
        assert_eq!(locations.len(), 1);
        assert_eq!(*locations["1"], station!(main_station));
    }

    #[test]
    fn test_process_child_without_parent() {
        let mut locations = HashMap::new();
        let record = Importer::process_record(main_station_platform_record(), &mut locations).unwrap_err();
        assert_eq!(record, main_station_platform_record());
        assert!(locations.is_empty());
    }

    #[test]
    fn test_process_child_with_parent() {
        let mut locations = HashMap::new();
        locations.insert("1".to_string(), Rc::new(station!(main_station)));
        Importer::process_record(main_station_platform_record(), &mut locations).unwrap();
        assert_eq!(locations.len(), 2);
        assert_eq!(*locations["2"], station!(main_station));
    }

    #[test]
    fn test_station_with_parent() {
        let mut dataset = crate::dataset!(
            stops:
                stop_id, stop_name,      stop_lat, stop_lon, location_type, parent_station;
                1,       "Main Station", 52.526,   13.369,   1,             10
        );

        let error = Importer::import(&mut dataset).unwrap_err();
        assert_eq!(format!("{}", error), "forbidden parent 10 for station 1");
    }

    #[test]
    fn test_child_missing_parent() {
        let mut dataset = crate::dataset!(
            stops:
                stop_id, stop_name,                 stop_lat, stop_lon, location_type, parent_station;
                2,       "Main Station Platform 1", 52.526,   13.369,   0,             1
        );

        let error = Importer::import(&mut dataset).unwrap_err();
        assert_eq!(format!("{}", error), "parent 1 for location 2 not found");
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = crate::dataset!(
            stops:
                stop_id, stop_name,      stop_lat, stop_lon, location_type, parent_station;
                1,       "Main Station", 52.526,   13.369,   1,             "";
                2,       "Center",       52.520,   13.387,   1,             ""
        );

        let locations = Importer::import(&mut dataset).unwrap();
        assert_eq!(locations.len(), 2);
        assert_eq!(*locations["1"], station!(main_station));
        assert_eq!(*locations["2"], station!(center));
    }
}
