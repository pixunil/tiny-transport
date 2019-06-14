use std::error::Error;
use std::rc::Rc;
use std::fmt;
use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserializer;
use serde::de::{Deserialize, Visitor, Error as DeserializeError};

use chrono::prelude::*;

use super::utils::*;
use super::location::Location;
use super::trip::Route;
use simulation::Color;

#[derive(Debug)]
pub struct Line {
    name: String,
    color: Option<Color>,
    pub kind: LineKind,
    pub routes: Vec<Route>,
}

impl Line {
    fn new(record: LineRecord) -> (Id, Line) {
        let line = Line {
            name: record.route_short_name,
            color: None,
            kind: record.route_type,
            routes: Vec::new(),
        };
        (record.agency_id, line)
    }

    fn add_routes(&mut self, routes: Option<Vec<Route>>) {
        routes.map(|routes| self.routes.extend(routes));
    }

    fn add_color_when_applicable(&mut self, colors: &HashMap<String, Color>) {
        match self.kind {
            LineKind::Railway | LineKind::SuburbanRailway | LineKind::UrbanRailway
              => self.color = colors.get(&self.name).cloned(),
            _ => {},
        }
    }

    pub fn into_line(&self, stations: &[Rc<Location>], date: &NaiveDate) -> (Color, simulation::IndexedLine) {
        let route = self.routes.iter()
            .max_by_key(|route| route.num_trips_at(date))
            .unwrap();
        let stops = route.into_stops(stations);
        let trains = route.into_trains(date);
        let color = self.color.clone().unwrap();
        (color, simulation::IndexedLine::new(self.name.clone(), stops, trains))
    }
}

pub fn from_csv(path: &mut PathBuf, mut routes: HashMap<Id, Vec<Route>>) -> Result<HashMap<Id, Vec<Line>>, Box<Error>> {
    let mut colors = HashMap::new();

    path.set_file_name("colors.csv");
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .from_path(&path)?;
    for result in reader.deserialize() {
        let record: LineColorRecord = result?;
        colors.insert(record.line, record.color);
    }

    let mut lines = HashMap::new();

    path.set_file_name("routes.txt");
    let mut reader = csv::Reader::from_path(&path)?;
    for result in reader.deserialize() {
        let record: LineRecord = result?;
        let key = (record.agency_id.clone(), record.route_short_name.clone(), record.route_type.clone());
        let id = record.route_id.clone();
        let (_agency_id, ref mut line) = lines.entry(key)
            .or_insert_with(|| Line::new(record));
        line.add_routes(routes.remove(&id));
        line.add_color_when_applicable(&colors);
    }

    let mut agency_lines = HashMap::new();
    for (_key, (agency_id, line)) in lines {
        agency_lines.entry(agency_id)
            .or_insert_with(Vec::new)
            .push(line);
    }

    Ok(agency_lines)
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum LineKind {
    Railway,
    SuburbanRailway,
    UrbanRailway,
    Bus,
    Tram,
    WaterTransport,
}

impl<'de> Deserialize<'de> for LineKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct LineKindVisitor;

        impl<'de> Visitor<'de> for LineKindVisitor {
            type Value = LineKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("positive integer")
            }

            fn visit_u64<E>(self, value: u64) -> Result<LineKind, E>
                where E: DeserializeError
            {
                match value {
                    100 => Ok(LineKind::Railway),
                    109 => Ok(LineKind::SuburbanRailway),
                    400 => Ok(LineKind::UrbanRailway),
                    3 | 700 => Ok(LineKind::Bus),
                    900 => Ok(LineKind::Tram),
                    1000 => Ok(LineKind::WaterTransport),
                    _ => Err(E::custom(format!("unknown route kind of value: {}", value))),
                }
            }
        }

        deserializer.deserialize_u64(LineKindVisitor)
    }
}

#[derive(Debug, Deserialize)]
struct LineColorRecord {
    line: String,
    #[serde(deserialize_with = "deserialize_color")]
    color: Color,
}

#[derive(Debug, Deserialize)]
struct LineRecord {
    route_id: Id,
    agency_id: Id,
    route_short_name: String,
    route_type: LineKind,
}
