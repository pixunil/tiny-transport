use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::io::{self, Write};

use clap::arg_enum;
use itertools::Itertools;

use import::coord::{project, project_back, transform, Point};
use import::line::Line;
use import::trip::Route;
use import::ImportedDataset;
use simulation::{Direction, Directions};

arg_enum! {
    #[derive(Debug, Clone, Copy)]
    pub enum Format {
        Import,
        Storage,
        Simulation,
    }
}

impl Format {
    fn format_position(self, (lat, lon): (f64, f64)) -> String {
        match self {
            Self::Import => format!("{:.3}, {:.3}", lat, lon),
            Self::Storage | Self::Simulation => {
                let transformed = transform(project(lat, lon));
                format!("{:6}, {:6}", transformed.x, transformed.y)
            }
        }
    }
}

fn round_position(position: Point) -> (f64, f64) {
    let (lat, lon) = project_back(position);
    (
        (lat * 1000.0).round() / 1000.0,
        (lon * 1000.0).round() / 1000.0,
    )
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

struct DirectionVec<T> {
    upstream: Vec<T>,
    downstream: Vec<T>,
}

impl<T: Clone> DirectionVec<T> {
    fn new() -> Self {
        Self {
            upstream: Vec::new(),
            downstream: Vec::new(),
        }
    }

    fn push(&mut self, directions: Directions, value: T) {
        if directions.allows(Direction::Upstream) {
            self.upstream.push(value.clone());
        }
        if directions.allows(Direction::Downstream) {
            self.downstream.push(value);
        }
    }
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
    in_directions: Directions,
    location_identifier: Option<String>,
}

struct NodeDisplay(Node, Format);

impl fmt::Display for NodeDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let direction_name = match self.0.in_directions {
            Directions::UpstreamOnly => "UpstreamOnly",
            Directions::DownstreamOnly => "DownstreamOnly",
            Directions::Both => "Both",
        };
        write!(
            formatter,
            "{}, {}",
            self.1.format_position(self.0.position),
            direction_name
        )?;
        if let Some(location_identifier) = &self.0.location_identifier {
            let padding = " ".repeat(14 - direction_name.len());
            write!(formatter, ",{} {};", padding, location_identifier)
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

fn inspect_route(
    output: &mut impl Write,
    format: Format,
    route: &Route,
    locations: &mut Vec<Location>,
    location_identifiers: &mut HashSet<String>,
) -> io::Result<()> {
    let mut stop_locations = DirectionVec::new();
    let mut shapes = DirectionVec::new();
    let mut nodes = Vec::new();

    for node in route.nodes() {
        let mut location_identifier = None;

        if let Some(location) = &node.location() {
            let name = normalize_name(location.name());
            let identifier = make_identifier(name);

            if location_identifiers.insert(identifier.clone()) {
                locations.push(Location {
                    position: round_position(location.position()),
                    name: name.to_string(),
                    identifier: identifier.clone(),
                });
            }
            stop_locations.push(node.in_directions(), identifier.clone());
            location_identifier = Some(identifier);
        }

        let position = round_position(node.position());
        nodes.push(Node {
            position,
            in_directions: node.in_directions(),
            location_identifier,
        });
        shapes.push(node.in_directions(), position);
    }

    let stop_locations_display = StopLocationsDisplay(stop_locations.upstream, format);
    writeln!(output, "{}", stop_locations_display)?;
    writeln!(output, "{}", ShapeDisplay(shapes.upstream, format))?;
    stop_locations.downstream.reverse();
    shapes.downstream.reverse();
    let stop_locations_display = StopLocationsDisplay(stop_locations.downstream, format);
    writeln!(output, "{}", stop_locations_display)?;
    writeln!(output, "{}", ShapeDisplay(shapes.downstream, format))?;

    nodes.dedup();
    for node in nodes {
        writeln!(output, "{}", NodeDisplay(node, format))?;
    }
    Ok(())
}

fn inspect_line(output: &mut impl Write, format: Format, line: &Line) -> io::Result<()> {
    let mut locations = Vec::new();
    let mut location_identifiers = HashSet::new();

    for route in line.routes() {
        inspect_route(
            output,
            format,
            route,
            &mut locations,
            &mut location_identifiers,
        )?;
        writeln!(output)?;
    }

    for location in locations {
        writeln!(output, "{}", LocationDisplay(location, format))?;
    }
    Ok(())
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

    for line in lines {
        inspect_line(output, format, line)?;
        writeln!(output)?;
    }
    Ok(())
}
