use std::error::Error;
use std::rc::Rc;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::collections::{VecDeque, HashMap};

use na::Point2;

use super::utils::*;

#[derive(Debug, PartialEq)]
pub struct Location {
    pub id: Id,
    pub name: String,
    lat: f32,
    lon: f32,
}

impl Location {
    fn new(record: LocationRecord) -> (Id, Location) {
        let location = Location {
            id: record.stop_id.clone(),
            name: record.stop_name,
            lat: record.stop_lat,
            lon: record.stop_lon,
        };
        (record.stop_id, location)
    }

    pub fn position(&self) -> Point2<f32> {
        let x = 2000.0 * (self.lon - 13.5);
        let y = -4000.0 * (self.lat - 52.52);
        Point2::new(x, y)
    }

    pub fn station_cmp(&self, other: &Location) -> Ordering {
        self.id.cmp(&other.id)
    }

    pub fn freeze(&self) -> serialization::Station {
        serialization::Station::new(self.position(), self.name.clone())
    }
}

pub struct Path {
    locations: Vec<Rc<Location>>,
}

impl Path {
    pub fn new(locations: Vec<Rc<Location>>) -> Path {
        Path { locations }
    }
}

impl Into<Vec<Rc<Location>>> for Path {
    fn into(self) -> Vec<Rc<Location>> {
        self.locations
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Path) -> bool {
        self.locations.iter().zip(&other.locations)
            .all(|(a, b)| a.id == b.id)
    }
}

impl Eq for Path {}

impl Hash for Path {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        for location in &self.locations {
            location.id.hash(hasher);
        }
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
            let (id, location) = Location::new(record);
            locations.insert(id, Rc::new(location));
        },
    }
}

pub fn from_csv(dataset: &mut impl Dataset) -> Result<HashMap<Id, Rc<Location>>, Box<dyn Error>> {
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

#[derive(Debug, PartialEq, Deserialize)]
struct LocationRecord {
    stop_id: Id,
    parent_station: Option<Id>,
    stop_name: String,
    stop_lat: f32,
    stop_lon: f32,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn main_station_record() -> LocationRecord {
        LocationRecord {
            stop_id: "1".into(),
            parent_station: None,
            stop_name: "Main Station".into(),
            stop_lat: 52.52,
            stop_lon: 13.37,
        }
    }

    pub fn main_station() -> Location {
        Location {
            id: "1".into(),
            name: "Main Station".into(),
            lat: 52.52,
            lon: 13.37,
        }
    }

    fn main_station_platform_record() -> LocationRecord {
        LocationRecord {
            stop_id: "2".into(),
            parent_station: Some("1".into()),
            stop_name: "Main Station Platform 1".into(),
            stop_lat: 52.52,
            stop_lon: 13.37,
        }
    }

    pub fn museum() -> Location {
        Location {
            id: "2".into(),
            name: "Museum".into(),
            lat: 52.53,
            lon: 13.38,
        }
    }

    #[test]
    fn test_import_location() {
        let (id, location) = Location::new(main_station_record());
        assert_eq!(id, "1");
        assert_eq!(location, main_station());
    }

    #[test]
    fn test_process_parent() {
        let mut queue = VecDeque::new();
        let mut locations = HashMap::new();
        process_record(main_station_record(), &mut queue, &mut locations);
        assert!(queue.is_empty());
        assert_eq!(locations.len(), 1);
        assert_eq!(*locations["1"], main_station());
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
        locations.insert("1".into(), Rc::new(main_station()));
        process_record(main_station_platform_record(), &mut queue, &mut locations);
        assert!(queue.is_empty());
        assert_eq!(locations.len(), 2);
        assert_eq!(*locations["2"], main_station());
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = crate::dataset!(
            stops:
                stop_id, stop_name,      stop_lat, stop_lon, parent_station;
                1,       "Main Station", 52.52,    13.37,    ""

        );

        let locations = from_csv(&mut dataset).unwrap();
        assert_eq!(locations.len(), 1);
        assert_eq!(*locations["1"], main_station());
    }
}
