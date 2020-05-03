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

    let line_groups: Vec<_> = dataset
        .line_groups
        .into_iter()
        .map(storage::LineGroup::load)
        .collect();

    println!(
        "Loaded {} stations and {} line groups",
        stations.len(),
        line_groups.len()
    );
    Ok(())
}
