use std::error::Error;
use std::fmt;
use std::collections::HashMap;

use serde::Deserializer;
use serde::de::{Deserialize, Visitor, Error as DeserializeError};

use chrono::prelude::*;

use crate::utils::*;
use crate::trip::Route;
use simulation::Color;

#[derive(Debug, PartialEq)]
pub(crate) struct Line {
    name: String,
    color: Color,
    pub(crate) kind: LineKind,
    pub(crate) routes: Vec<Route>,
}

impl Line {
    pub(crate) fn new(name: String, kind: LineKind) -> Line {
        Line {
            name,
            color: kind.color(),
            kind,
            routes: Vec::new()
        }
    }

    fn add_routes(&mut self, routes: Option<Vec<Route>>) {
        if let Some(routes) = routes {
            self.routes.extend(routes)
        }
    }

    fn add_color_when_applicable(&mut self, colors: &HashMap<String, Color>) {
        match self.kind {
            LineKind::Railway | LineKind::SuburbanRailway | LineKind::UrbanRailway => {
                if let Some(color) = colors.get(&self.name).cloned() {
                    self.color = color;
                }
            },
            _ => {},
        }
    }

    pub(crate) fn freeze(&self, date: NaiveDate) -> (Color, serialization::Line) {
        let route = self.routes.iter()
            .max_by_key(|route| route.num_trips_at(date))
            .unwrap();
        let nodes = route.freeze_nodes();
        let trains = route.freeze_trains(date);
        let color = self.color.clone();
        (color, serialization::Line::new(self.name.clone(), nodes, trains))
    }
}

impl From<LineRecord> for Line {
    fn from(record: LineRecord) -> Line {
        Line::new(record.route_short_name, record.route_type)
    }
}

pub(crate) struct Importer {
    records: Vec<LineRecord>,
    id_mapping: HashMap<Id, usize>,
    colors: HashMap<String, Color>,
}

impl Importer {
    pub(crate) fn import(dataset: &mut impl Dataset) -> Result<Importer, Box<dyn Error>> {
        let (records, id_mapping) = Self::import_lines(dataset)?;
        let colors = Self::import_colors(dataset)?;
        Ok(Importer { records, id_mapping, colors })
    }

    fn import_lines(dataset: &mut impl Dataset)
        -> Result<(Vec<LineRecord>, HashMap<Id, usize>), Box<dyn Error>>
    {
        let mut deduplicated_records = HashMap::new();
        let mut reader = dataset.read_csv("routes.txt")?;
        for result in reader.deserialize() {
            let record: LineRecord = result?;
            let key = (record.agency_id.clone(), record.route_short_name.clone(), record.route_type);
            let id = record.route_id.clone();
            let (_, ids) = deduplicated_records.entry(key)
                .or_insert_with(|| (record, Vec::new()));
            ids.push(id);
        }

        let mut records = Vec::new();
        let mut id_mapping = HashMap::new();
        for (_key, (record, ids)) in deduplicated_records {
            id_mapping.extend(ids.into_iter().map(|id| (id, records.len())));
            records.push(record);
        }
        Ok((records, id_mapping))
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

    pub(crate) fn id_mapping(&self) -> &HashMap<Id, usize> {
        &self.id_mapping
    }

    pub(crate) fn num_lines(&self) -> usize {
        self.records.len()
    }

    pub(crate) fn add_routes(self, mut routes: Vec<Vec<Route>>) -> Result<HashMap<Id, Vec<Line>>, Box<dyn Error>> {
        let mut lines = HashMap::new();
        for record in self.records.into_iter().rev() {
            let agency_id = record.agency_id.clone();
            let mut line = Line::from(record);
            line.add_routes(routes.pop());
            line.add_color_when_applicable(&self.colors);
            lines.entry(agency_id)
                .or_insert_with(Vec::new)
                .push(line);
        }

        Ok(lines)
    }
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

impl LineKind {
    fn color(self) -> Color {
        match self {
            LineKind::Railway => Color::new(227, 0, 27),
            LineKind::SuburbanRailway => Color::new(0, 114, 56),
            LineKind::UrbanRailway => Color::new(0, 100, 173),
            LineKind::Bus => Color::new(125, 23, 107),
            LineKind::Tram => Color::new(204, 10, 34),
            LineKind::WaterTransport => Color::new(0, 128, 186),
        }
    }
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
    #[serde(deserialize_with = "deserialize::color")]
    color: Color,
}

#[derive(Debug, Deserialize)]
struct LineRecord {
    route_id: Id,
    agency_id: Id,
    route_short_name: String,
    route_type: LineKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_test::{Token, assert_de_tokens, assert_de_tokens_error};
    use crate::dataset;

    #[macro_export]
    macro_rules! line_ {
        ($name:expr, $kind:ident) => (
            $crate::line::Line::new($name.to_string(), $crate::line::LineKind::$kind)
        );
        (blue) => (
            $crate::line_!("Blue Line", SuburbanRailway)
        );
        (blue-replacement) => (
            $crate::line_!("Blue Line", Bus)
        );
    }

    fn colors() -> HashMap<String, Color> {
        let mut colors = HashMap::new();
        colors.insert("Blue Line".to_string(), Color::new(0, 0, 255));
        colors
    }

    #[test]
    fn test_import_line() {
        let record = LineRecord {
            route_id: "1".to_string(),
            agency_id: "1".to_string(),
            route_short_name: "Blue Line".to_string(),
            route_type: LineKind::SuburbanRailway,
        };
        assert_eq!(Line::from(record), line_!(blue));
    }

    #[test]
    fn test_deduplication() {
        let mut dataset = dataset!(
            routes:
                route_id, agency_id, route_short_name, route_type;
                1,        1,         "Blue Line",      109;
                2,        1,         "Blue Line",      109
        );

        let (_, id_mapping) = Importer::import_lines(&mut dataset).unwrap();
        assert_eq!(id_mapping.len(), 2);
        assert_eq!(id_mapping["1"], 0);
        assert_eq!(id_mapping["2"], 0);
    }

    #[test]
    fn test_add_color_to_applicable() {
        let mut line = line_!(blue);
        line.add_color_when_applicable(&colors());
        assert_eq!(line.color, Color::new(0, 0, 255));
    }

    #[test]
    fn test_add_color_to_unapplicable() {
        let mut line = line_!(blue-replacement);
        line.add_color_when_applicable(&colors());
        assert_eq!(line.color, LineKind::Bus.color());
    }

    #[test]
    fn test_deserialize_line_kind() {
        assert_de_tokens(&LineKind::Railway, &[Token::U16(100)]);
        assert_de_tokens(&LineKind::SuburbanRailway, &[Token::U16(109)]);
        assert_de_tokens(&LineKind::UrbanRailway, &[Token::U16(400)]);
        assert_de_tokens(&LineKind::Bus, &[Token::U16(3)]);
        assert_de_tokens(&LineKind::Bus, &[Token::U16(700)]);
        assert_de_tokens(&LineKind::Tram, &[Token::U16(900)]);
        assert_de_tokens(&LineKind::WaterTransport, &[Token::U16(1000)]);
        assert_de_tokens_error::<LineKind>(&[Token::U16(0)],
            "unknown route kind of value: 0");
        assert_de_tokens_error::<LineKind>(&[Token::Str("")],
            "invalid type: string \"\", expected positive integer");
    }
}
