use std::error::Error;
use std::rc::Rc;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::collections::{VecDeque, HashMap};

use na::Point2;

use super::utils::*;

#[derive(Debug)]
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

    pub fn freeze(&self) -> serialization::Station {
        let x = 2000.0 * (self.lon - 13.5);
        let y = -4000.0 * (self.lat - 52.52);
        serialization::Station::new(Point2::new(x, y), self.name.clone())
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Location) -> bool {
        self.id == other.id
    }
}

impl Eq for Location {}

impl Ord for Location {
    fn cmp(&self, other: &Location) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Location) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Location {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
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

#[derive(Debug, Deserialize)]
struct LocationRecord {
    stop_id: Id,
    parent_station: Option<Id>,
    stop_name: String,
    stop_lat: f32,
    stop_lon: f32,
}
