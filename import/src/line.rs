use std::error::Error;
use std::rc::Rc;
use std::fmt;
use std::collections::HashMap;

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
    fn new(record: LineRecord) -> Line {
        Line {
            name: record.route_short_name,
            color: None,
            kind: record.route_type,
            routes: Vec::new(),
        }
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

    pub fn freeze(&self, stations: &[Rc<Location>], date: &NaiveDate) -> (Color, serialization::Line) {
        let route = self.routes.iter()
            .max_by_key(|route| route.num_trips_at(date))
            .unwrap();
        let stops = route.freeze_stops(stations);
        let trains = route.freeze_trains(date);
        let color = self.color.clone().unwrap();
        (color, serialization::Line::new(self.name.clone(), stops, trains))
    }
}

fn import_colors(dataset: &mut impl Dataset) -> Result<HashMap<String, Color>, Box<dyn Error>> {
    let mut colors = HashMap::new();
    let mut reader = dataset.read_csv("colors.txt")?;
    for result in reader.deserialize() {
        let record: LineColorRecord = result?;
        colors.insert(record.line, record.color);
    }
    Ok(colors)
}

fn import_lines(dataset: &mut impl Dataset, mut routes: HashMap<Id, Vec<Route>>, colors: HashMap<String, Color>)
    -> Result<HashMap<Id, Vec<Line>>, Box<dyn Error>>
{
    let mut deduplicated_lines = HashMap::new();
    let mut reader = dataset.read_csv("routes.txt")?;
    for result in reader.deserialize() {
        let record: LineRecord = result?;
        let key = (record.agency_id.clone(), record.route_short_name.clone(), record.route_type.clone());
        let id = record.route_id.clone();
        let line = deduplicated_lines.entry(key)
            .or_insert_with(|| Line::new(record));
        line.add_routes(routes.remove(&id));
        line.add_color_when_applicable(&colors);
    }

    let mut lines = HashMap::new();
    for ((agency_id, _name, _kind), line) in deduplicated_lines {
        lines.entry(agency_id)
            .or_insert_with(Vec::new)
            .push(line);
    }

    Ok(lines)
}

pub fn from_csv(dataset: &mut impl Dataset, routes: HashMap<Id, Vec<Route>>) -> Result<HashMap<Id, Vec<Line>>, Box<dyn Error>> {
    let colors = import_colors(dataset)?;
    let lines = import_lines(dataset, routes, colors)?;
    Ok(lines)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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
