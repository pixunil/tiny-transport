use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsStr;

use clap::arg_enum;
use itertools::Itertools;

use import::coord::{project, project_back, transform, Point};
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

    fn print_route_info(self, stop_locations: Vec<String>, shape: Vec<(f64, f64)>) {
        if !stop_locations.is_empty() {
            let termini = (
                stop_locations.first().unwrap(),
                stop_locations.last().unwrap(),
            );
            println!("{}_{}", termini.0, termini.1);
        } else {
            println!("‹no stop locations›");
        }
        println!("[{}]", stop_locations.iter().format(", "));
        match self {
            Self::Import => self.print_shape(shape),
            Self::Storage | Self::Simulation => {}
        }
    }

    fn print_shape(self, shape: Vec<(f64, f64)>) {
        println!(
            "[{}]",
            shape
                .iter()
                .dedup()
                .format_with("; ", |&point, f| f(&self.format_position(point)))
        );
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

#[derive(PartialEq)]
struct Node {
    position: (f64, f64),
    in_directions: Directions,
    location_identifier: Option<String>,
}

impl Node {
    fn print(&self, format: Format) {
        let direction_name = match self.in_directions {
            Directions::UpstreamOnly => "UpstreamOnly",
            Directions::DownstreamOnly => "DownstreamOnly",
            Directions::Both => "Both",
        };
        print!(
            "{}, {}",
            format.format_position(self.position),
            direction_name
        );
        if let Some(location_identifier) = &self.location_identifier {
            let padding = " ".repeat(14 - direction_name.len());
            println!(",{} {};", padding, location_identifier);
        } else {
            println!(";");
        }
    }
}

struct Location {
    position: (f64, f64),
    name: String,
    identifier: String,
}

impl Location {
    fn print(&self, format: Format) {
        let padding = " ".repeat(36 - self.identifier.len().min(36));
        println!(
            "{}:{} {}, \"{}\";",
            self.identifier,
            padding,
            format.format_position(self.position),
            self.name,
        );
    }
}

pub(crate) fn inspect(
    dataset: impl AsRef<OsStr>,
    line_name: &str,
    format: Format,
) -> Result<(), Box<dyn Error>> {
    let imported = ImportedDataset::import(dataset)?;

    let line = imported
        .agencies()
        .filter(|agency| agency.name() == "Berliner Verkehrsbetriebe")
        .flat_map(|agency| agency.lines())
        .find(|line| line.name() == line_name)
        .ok_or_else(|| format!("No line with name {} found.", line_name))?;

    let mut locations = Vec::new();
    let mut location_identifiers = HashSet::new();

    for route in line.routes() {
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

        format.print_route_info(stop_locations.upstream, shapes.upstream);
        stop_locations.downstream.reverse();
        shapes.downstream.reverse();
        format.print_route_info(stop_locations.downstream, shapes.downstream);

        nodes.dedup();
        for node in nodes {
            node.print(format)
        }

        println!();
    }

    for location in locations {
        location.print(format);
    }

    Ok(())
}
