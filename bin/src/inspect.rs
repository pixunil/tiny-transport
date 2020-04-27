use std::collections::HashSet;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::{self, Formatter};

use itertools::Itertools;

use import::coord::project_back;
use import::ImportedDataset;
use simulation::{Direction, Directions};

fn normalize_name(mut name: &str) -> &str {
    const REMOVE_FROM_START: &[&str] = &["Berlin,", "S+U", "S", "U"];
    const REMOVE_FROM_END: &[&str] = &["(Berlin)"];

    for remove in REMOVE_FROM_START {
        name = name.trim_start_matches(remove);
    }
    for remove in REMOVE_FROM_END {
        name = name.trim_end_matches(remove);
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

struct Node {
    position: (f64, f64),
    in_directions: Directions,
    location_identifier: Option<String>,
}

impl fmt::Display for Node {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let direction_name = match self.in_directions {
            Directions::UpstreamOnly => "UpstreamOnly",
            Directions::DownstreamOnly => "DownstreamOnly",
            Directions::Both => "Both",
        };
        let (lat, lon) = self.position;
        write!(formatter, "{:.3}, {:.3}, {}", lat, lon, direction_name)?;
        if let Some(location_identifier) = &self.location_identifier {
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

impl fmt::Display for Location {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let (lat, lon) = self.position;
        let padding = " ".repeat(36 - self.identifier.len().min(36));
        write!(
            formatter,
            "{}:{} {:.3}, {:.3}, \"{}\";",
            self.identifier, padding, lat, lon, self.name,
        )
    }
}

fn print_route_info(stop_locations: Vec<String>, shapes: Vec<(f64, f64)>) {
    if stop_locations.len() > 0 {
        let termini = (
            stop_locations.first().unwrap(),
            stop_locations.last().unwrap(),
        );
        println!("{}_{}", termini.0, termini.1);
    } else {
        println!("‹no stop locations›");
    }
    println!(
        "[{}]",
        stop_locations
            .iter()
            .format_with(" ", |location, f| f(&format_args!("{},", location)))
    );
    println!(
        "[{}]",
        shapes.iter().format_with(" ", |point, f| f(&format_args!(
            "{:.3}, {:.3};",
            point.0, point.1
        )))
    );
}

pub(crate) fn inspect(dataset: impl AsRef<OsStr>, line_name: &str) -> Result<(), Box<dyn Error>> {
    let imported = ImportedDataset::import(dataset)?;

    let line = imported
        .agencies()
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
                        position: project_back(location.position()),
                        name: name.to_string(),
                        identifier: identifier.clone(),
                    });
                }
                stop_locations.push(node.in_directions(), identifier.clone());
                location_identifier = Some(identifier);
            }

            nodes.push(Node {
                position: project_back(node.position()),
                in_directions: node.in_directions(),
                location_identifier,
            });
            shapes.push(node.in_directions(), project_back(node.position()));
        }

        print_route_info(stop_locations.upstream, shapes.upstream);
        stop_locations.downstream.reverse();
        shapes.downstream.reverse();
        print_route_info(stop_locations.downstream, shapes.downstream);

        nodes
            .iter()
            .map(|node| format!("{}", node))
            .dedup()
            .for_each(|node| println!("{}", node));

        println!();
    }

    for location in locations {
        println!("{}", location);
    }

    Ok(())
}
