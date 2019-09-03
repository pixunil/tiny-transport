use std::error::Error;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Write;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::fs::{self, File};

use chrono::prelude::*;

use zip::{CompressionMethod, ZipArchive, ZipWriter};
use zip::write::FileOptions;

mod utils;
mod service;
mod agency;
mod shape;
mod location;
mod line;
mod trip;

use utils::Dataset;
use agency::Agency;
use line::{Line, LineKind};

pub fn compress() -> Result<(), Box<dyn Error>> {
    let mut zip = ZipWriter::new(File::create("import/data/vbb.bzip")?);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Bzip2);

    for entry in fs::read_dir("import/data/vbb")? {
        let entry = entry?;
        zip.start_file_from_path(entry.file_name().as_ref(), options)?;

        let data = fs::read(entry.path())?;
        zip.write_all(&data)?;
    }

    zip.finish()?;
    Ok(())
}

fn fetch(mut dataset: impl Dataset) -> Result<Vec<Agency>, Box<dyn Error>> {
    let services = service::Importer::import(&mut dataset)?;
    let locations = location::Importer::import(&mut dataset)?;
    let shapes = shape::Importer::import(&mut dataset)?;
    let line_importer = line::Importer::import(&mut dataset)?;
    let trip_importer = trip::Importer::new(&services, &locations, &shapes, line_importer.id_mapping(), line_importer.num_lines());
    let routes = trip_importer.import(&mut dataset)?;
    let lines = line_importer.add_routes(routes)?;
    let agencies = agency::Importer::import(&mut dataset, lines)?;
    Ok(agencies)
}

fn filter(agencies: &[Agency]) -> impl Iterator<Item = &'_ Line> + Clone {
    let agency = agencies.iter()
        .find(|agency| agency.name() == "S-Bahn Berlin GmbH")
        .unwrap();

    agency.lines().iter()
        .filter(|line| line.kind == LineKind::SuburbanRailway)
}

fn store<'a>(lines: impl Iterator<Item = &'a Line> + Clone) -> Result<(), Box<dyn Error>> {
    let mut stations = lines.clone()
        .flat_map(|line| &line.routes)
        .flat_map(|route| &route.locations)
        .cloned()
        .collect::<Vec<_>>();
    stations.sort_unstable_by(|a, b| a.station_cmp(b));
    stations.dedup_by(|a, b| a.station_cmp(b) == Ordering::Equal);

    let date = NaiveDate::from_ymd(2019, 8, 26);

    let mut line_groups = HashMap::new();
    for line in lines {
        let (color, line) = line.freeze(date);
        line_groups.entry(color.clone())
            .or_insert_with(Vec::new)
            .push(line);
    }

    let stations = stations.into_iter()
        .map(|station| station.freeze())
        .collect();

    let line_groups = line_groups.into_iter()
        .map(|(color, line_group)| serialization::LineGroup::new(color, line_group))
        .collect();

    let dataset = serialization::Dataset::new(stations, line_groups);
    let file = File::create("wasm/www/data.bin")?;
    bincode::serialize_into(file, &dataset)?;
    Ok(())
}

pub fn import(path: impl AsRef<OsStr>) -> Result<(), Box<Error>> {
    let path = Path::new(&path);
    let agencies = if path.is_dir() {
        let mut path = PathBuf::from(&path);
        path.push(".txt");
        fetch(path)?
    } else {
        let archive = ZipArchive::new(File::open(&path)?)?;
        fetch(archive)?
    };
    let lines = filter(&agencies);
    store(lines)
}
