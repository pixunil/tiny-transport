use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub(crate) fn load(data: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(data)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let dataset = bincode::deserialize::<storage::Dataset>(&data).unwrap();

    let stations = dataset
        .stations
        .into_iter()
        .map(storage::Station::load)
        .collect::<Vec<_>>();

    let lines = dataset
        .lines
        .into_iter()
        .map(storage::Line::load)
        .collect::<Vec<_>>();

    println!(
        "Loaded {} stations and {} lines",
        stations.len(),
        lines.len(),
    );
    Ok(())
}
