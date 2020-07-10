use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub(crate) fn load(binary: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(binary)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let dataset = bincode::deserialize::<storage::Dataset>(&data)
        .unwrap()
        .load();

    println!(
        "Loaded {} stations and {} lines",
        dataset.station_count(),
        dataset.line_count(),
    );
    Ok(())
}
