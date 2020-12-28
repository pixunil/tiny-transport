use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::io::{self, Write};

use clap::arg_enum;
use itertools::Itertools;

use import::coord::{project, project_back, transform, Point};
use import::line::Line;
use import::path::Segment;
use import::trip::Route;
use import::ImportedDataset;

arg_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum Format {
        Import,
        ImportLong,
        Storage,
        Simulation,
    }
}

impl Format {
    fn round_position(self, position: Point) -> (f64, f64) {
        fn round(value: f64, factor: f64) -> f64 {
            (value * factor).round() / factor
        }

        let (lat, lon) = project_back(position);
        match self {
            Self::Import | Self::Storage | Self::Simulation => {
                let factor = 1000.0;
                (round(lat, factor), round(lon, factor))
            }
            Self::ImportLong => (lat, lon),
        }
    }

    fn format_position(self, (lat, lon): (f64, f64)) -> String {
        match self {
            Self::Import => format!("{:.3}, {:.3}", lat, lon),
            Self::ImportLong => format!("{:.6}, {:.6}", lat, lon),
            Self::Storage | Self::Simulation => {
                let transformed = transform(project(lat, lon));
                format!("{:6}, {:6}", transformed.x, transformed.y)
            }
        }
    }

    fn should_dedup(self) -> bool {
        match self {
            Self::Import | Self::Storage | Self::Simulation => true,
            Self::ImportLong => false,
        }
    }
}

fn normalize_name(mut name: &str) -> &str {
    const REMOVE_FROM_START: &[&str] = &["Berlin,", "S+U", "S", "U"];
    const REMOVE_FROM_END: &[&str] = &["(Berlin)", "Bhf"];

    for remove in REMOVE_FROM_START {
        name = name.trim_start_matches(remove);
    }
    for remove in REMOVE_FROM_END {
        name = name.trim_end_matches(remove).trim();
    }
    name.trim()
}

fn make_identifier(name: &str) -> String {
    name.to_lowercase()
        .replace('ä', "ae")
        .replace('ö', "oe")
        .replace('ü', "ue")
        .replace('ß', "ss")
        .replace(|character: char| !character.is_alphanumeric(), "_")
        .replace("__", "_")
        .trim_matches('_')
        .to_string()
}

struct ShapeDisplay(Vec<(f64, f64)>, Format);

impl fmt::Display for ShapeDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.1 {
            Format::Import => write!(
                formatter,
                "[{}]",
                self.0
                    .iter()
                    .dedup()
                    .format_with("; ", |&point, f| f(&self.1.format_position(point)))
            ),
            Format::ImportLong => write!(
                formatter,
                "[{}]",
                self.0
                    .iter()
                    .format_with("; ", |&point, f| f(&self.1.format_position(point)))
            ),
            Format::Simulation | Format::Storage => Ok(()),
        }
    }
}

struct StopLocationsDisplay(Vec<String>, Format);

impl fmt::Display for StopLocationsDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if !self.0.is_empty() {
            let termini = (self.0.first().unwrap(), self.0.last().unwrap());
            writeln!(formatter, "{}_{}", termini.0, termini.1)?;
        } else {
            writeln!(formatter, "‹no stop locations›")?;
        }
        write!(formatter, "[{}]", self.0.iter().format(", "))
    }
}

#[derive(PartialEq)]
struct Node {
    position: (f64, f64),
    location_identifier: Option<String>,
}

struct NodeDisplay(Node, Format);

impl fmt::Display for NodeDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.1.format_position(self.0.position),)?;
        if let Some(location_identifier) = &self.0.location_identifier {
            write!(formatter, ", {};", location_identifier)
        } else {
            write!(formatter, ";")
        }
    }
}

struct Location {
    position: (f64, f64),
    name: String,
    identifier: String,
}

struct LocationDisplay(Location, Format);

impl fmt::Display for LocationDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let padding = " ".repeat(36 - self.0.identifier.len().min(36));
        write!(
            formatter,
            "{}:{} {}, \"{}\";",
            self.0.identifier,
            padding,
            self.1.format_position(self.0.position),
            self.0.name,
        )
    }
}

struct Inspector<'a, W> {
    format: Format,
    output: &'a mut W,
}

impl<'a, W: Write> Inspector<'a, W> {
    fn new(format: Format, output: &'a mut W) -> Self {
        Self { format, output }
    }

    fn write(&mut self, value: impl fmt::Display) -> io::Result<()> {
        writeln!(self.output, "{}", value)
    }

    fn inspect_route(
        &mut self,
        route: &Route,
        segments: &[Segment],
        locations: &mut Vec<Location>,
        location_identifiers: &mut HashSet<String>,
    ) -> io::Result<()> {
        let mut stop_locations = Vec::new();
        let mut shapes = Vec::new();
        let mut nodes = Vec::new();

        for node in route.path().nodes(segments) {
            let mut location_identifier = None;

            if let Some(location) = &node.location() {
                let name = normalize_name(location.name());
                let identifier = make_identifier(name);

                if location_identifiers.insert(identifier.clone()) {
                    locations.push(Location {
                        position: self.format.round_position(location.position()),
                        name: name.to_string(),
                        identifier: identifier.clone(),
                    });
                }
                stop_locations.push(identifier.clone());
                location_identifier = Some(identifier);
            }

            let position = self.format.round_position(node.position());
            nodes.push(Node {
                position,
                location_identifier,
            });
            shapes.push(position);
        }

        self.write(StopLocationsDisplay(stop_locations, self.format))?;
        self.write(ShapeDisplay(shapes, self.format))?;

        if self.format.should_dedup() {
            nodes.dedup();
        }
        for node in nodes {
            self.write(NodeDisplay(node, self.format))?;
        }
        Ok(())
    }

    fn inspect_line(&mut self, line: &Line, segments: &[Segment]) -> io::Result<()> {
        let mut locations = Vec::new();
        let mut location_identifiers = HashSet::new();

        for route in line.routes() {
            self.inspect_route(route, segments, &mut locations, &mut location_identifiers)?;
            writeln!(self.output)?;
        }

        for location in locations {
            self.write(LocationDisplay(location, self.format))?;
        }
        Ok(())
    }

    fn inspect_lines<'b>(
        &mut self,
        lines: impl Iterator<Item = &'b Line>,
        segments: &[Segment],
    ) -> io::Result<()> {
        for line in lines {
            self.inspect_line(line, segments)?;
            writeln!(self.output)?;
        }
        Ok(())
    }
}

pub(crate) fn inspect(
    dataset: &ImportedDataset,
    agency_name: Option<&str>,
    line_name: &str,
    format: Format,
    output: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    let lines = dataset
        .agencies()
        .filter(|agency| match agency_name {
            Some(agency_name) => agency.name().contains(agency_name),
            None => true,
        })
        .flat_map(|agency| agency.lines())
        .filter(|line| line.name() == line_name);

    let mut inspector = Inspector::new(format, output);
    inspector.inspect_lines(lines, dataset.segments())?;
    Ok(())
}
