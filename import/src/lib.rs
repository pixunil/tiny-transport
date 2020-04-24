use std::cmp::Ordering;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};

use chrono::prelude::*;

use zip::ZipArchive;

mod agency;
mod coord;
mod deserialize;
mod line;
mod location;
pub mod profile;
mod service;
mod shape;
mod trip;
mod utils;

use agency::Agency;
use line::Line;
use profile::Profile;
use utils::Dataset;

fn fetch(mut dataset: impl Dataset) -> Result<Vec<Agency>, Box<dyn Error>> {
    let services = service::Importer::import(&mut dataset)?;
    let locations = location::Importer::import(&mut dataset)?;
    let shapes = shape::Importer::import(&mut dataset)?;
    let line_importer = line::Importer::import(&mut dataset)?;
    let trip_importer = trip::Importer::new(
        &services,
        &locations,
        &shapes,
        line_importer.id_mapping(),
        line_importer.num_lines(),
    );
    let routes = trip_importer.import(&mut dataset)?;
    let lines = line_importer.finish(routes)?;
    let agencies = agency::Importer::import(&mut dataset, lines)?;
    Ok(agencies)
}

fn store(lines: Vec<&Line>) -> Result<(), Box<dyn Error>> {
    let mut stations = lines
        .iter()
        .flat_map(|line| &line.routes)
        .flat_map(|route| route.locations())
        .cloned()
        .collect::<Vec<_>>();
    stations.sort_unstable_by(|a, b| a.station_cmp(b));
    stations.dedup_by(|a, b| a.station_cmp(b) == Ordering::Equal);

    let date = NaiveDate::from_ymd(2019, 8, 26);

    let mut line_groups = HashMap::new();
    for line in lines {
        let (color, line) = line.freeze(date);
        line_groups
            .entry(color.clone())
            .or_insert_with(Vec::new)
            .push(line);
    }

    let stations = stations
        .into_iter()
        .map(|station| station.freeze())
        .collect();

    let line_groups = line_groups
        .into_iter()
        .map(|(color, line_group)| serialization::LineGroup::new(color, line_group))
        .collect();

    let dataset = serialization::Dataset::new(stations, line_groups);
    let file = File::create("wasm/www/data.bin")?;
    bincode::serialize_into(file, &dataset)?;
    Ok(())
}

pub fn import(path: impl AsRef<OsStr>, profile: Profile) -> Result<(), Box<dyn Error>> {
    let path = Path::new(&path);
    let agencies = if path.is_dir() {
        let mut path = PathBuf::from(&path);
        path.push(".txt");
        fetch(path)?
    } else {
        let archive = ZipArchive::new(File::open(&path)?)?;
        fetch(archive)?
    };
    let lines = profile.filter(agencies.iter());
    store(lines)
}
