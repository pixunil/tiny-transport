use std::error::Error;
use std::rc::Rc;
use std::cmp::Ordering;
use std::collections::{VecDeque, HashMap};

use na::Point2;

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

fn process_record(record: LocationRecord, queue: &mut VecDeque<LocationRecord>, locations: &mut HashMap<Id, Rc<Location>>) {
    match record.parent_station {
        Some(ref parent_id) => {
            match locations.get(parent_id).cloned() {
                Some(parent) => {
                    locations.insert(record.stop_id, parent);
                },
                None => {
                    queue.push_back(record);
                },
            }
        },
        None => {
            let id = record.stop_id.clone();
            locations.insert(id, Rc::new(Location::from(record)));
        },
    }
}

pub struct Importer;

impl Importer {
    pub fn import(dataset: &mut impl Dataset) -> Result<HashMap<Id, Rc<Location>>, Box<dyn Error>> {
        let mut queue = VecDeque::new();
        let mut locations = HashMap::new();
        let mut reader = dataset.read_csv("stops.txt")?;
        for result in reader.deserialize() {
            process_record(result?, &mut queue, &mut locations);
        }

        while let Some(record) = queue.pop_front() {
            process_record(record, &mut queue, &mut locations);
        }

        Ok(locations)
    }
}

#[derive(Debug, PartialEq, Deserialize)]
struct LocationRecord {
    stop_id: Id,
    parent_station: Option<Id>,
    stop_name: String,
    stop_lat: f32,
    stop_lon: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

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
            parent_station: None,
            stop_name: "Main Station".to_string(),
            stop_lat: 52.526,
            stop_lon: 13.369,
        }
    }

    fn main_station_platform_record() -> LocationRecord {
        LocationRecord {
            stop_id: "2".to_string(),
            parent_station: Some("1".to_string()),
            stop_name: "Main Station Platform 1".to_string(),
            stop_lat: 52.526,
            stop_lon: 13.369,
        }
    }

    #[test]
    fn test_import_location() {
        let location = Location::from(main_station_record());
        assert_eq!(location, station!(main_station));
    }

    #[test]
    fn test_process_parent() {
        let mut queue = VecDeque::new();
        let mut locations = HashMap::new();
        process_record(main_station_record(), &mut queue, &mut locations);
        assert!(queue.is_empty());
        assert_eq!(locations.len(), 1);
        assert_eq!(*locations["1"], station!(main_station));
    }

    #[test]
    fn test_process_child_without_parent() {
        let mut queue = VecDeque::new();
        let mut locations = HashMap::new();
        process_record(main_station_platform_record(), &mut queue, &mut locations);
        assert!(locations.is_empty());
        assert_eq!(queue, [main_station_platform_record()]);
    }

    #[test]
    fn test_process_child_with_parent() {
        let mut queue = VecDeque::new();
        let mut locations = HashMap::new();
        locations.insert("1".to_string(), Rc::new(station!(main_station)));
        process_record(main_station_platform_record(), &mut queue, &mut locations);
        assert!(queue.is_empty());
        assert_eq!(locations.len(), 2);
        assert_eq!(*locations["2"], station!(main_station));
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = crate::dataset!(
            stops:
                stop_id, stop_name,      stop_lat, stop_lon, parent_station;
                1,       "Main Station", 52.526,   13.369,   "";
                2,       "Center",       52.520,   13.387,   ""
        );

        let locations = Importer::import(&mut dataset).unwrap();
        assert_eq!(locations.len(), 2);
        assert_eq!(*locations["1"], station!(main_station));
        assert_eq!(*locations["2"], station!(center));
    }
}
