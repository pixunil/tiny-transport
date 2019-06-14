#[macro_use]
extern crate serde_derive;
extern crate gtfs_sim_simulation as simulation;

use std::error::Error;
use std::collections::{HashSet, HashMap};
use std::fs::File;
use std::path::PathBuf;

use chrono::prelude::*;

mod utils;
mod service;
mod agency;
mod location;
mod line;
mod trip;

use agency::Agency;
use line::{Line, LineKind};

fn import() -> Result<Vec<Agency>, Box<dyn Error>> {
    let mut path = PathBuf::from("import/data/vbb/csv");
    let services = service::from_csv(&mut path)?;
    let locations = location::from_csv(&mut path)?;
    let trips = trip::from_csv(&mut path, &services, &locations)?;
    let lines = line::from_csv(&mut path, trips)?;
    let agencies = agency::from_csv(&mut path, lines)?;
    Ok(agencies)
}

fn filter<'a>(agencies: &'a Vec<Agency>) -> impl Iterator<Item = &'a Line> + Clone {
    let agency = agencies.iter()
        .find(|agency| agency.name == "S-Bahn Berlin GmbH")
        .unwrap();

    agency.lines.iter()
        .filter(|line| line.kind == LineKind::SuburbanRailway)
}

fn store<'a, I>(lines: I) -> Result<(), Box<dyn Error>>
    where I: Iterator<Item = &'a Line> + Clone
{
    let stations = lines.clone()
        .flat_map(|line| &line.routes)
        .flat_map(|route| &route.locations)
        .collect::<HashSet<_>>()
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();

    let date = NaiveDate::from_ymd(2019, 8, 26);

    let mut line_groups = HashMap::new();
    for line in lines {
        let (color, line) = line.into_line(&stations, &date);
        line_groups.entry(color.clone())
            .or_insert_with(Vec::new)
            .push(line)
    }

    let stations = stations.into_iter()
        .enumerate()
        .map(|(id, station)| station.into_station(id))
        .collect();

    let line_groups = line_groups.into_iter()
        .map(|(color, line_group)| simulation::IndexedLineGroup::new(color, line_group))
        .collect();

    let dataset = simulation::Dataset::new(stations, line_groups);
    let file = File::create("wasm/www/data.bin")?;
    bincode::serialize_into(file, &dataset)?;
    Ok(())
}

fn main() {
    let agencies = match import() {
        Ok(agencies) => agencies,
        Err(error) => return println!("{}", error),
    };
    let lines = filter(&agencies);
    store(lines).unwrap();
}
