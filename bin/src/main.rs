use std::convert::TryFrom;
use std::error::Error;
use std::fs::File;

use chrono::NaiveDate;
use clap::clap_app;

use import::profile::Profile;
use import::ImportedDataset;

mod compress;
mod inspect;
mod load;

use compress::compress;
use inspect::inspect;
use load::load;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app!(gtfs_sim =>
        (@subcommand compress =>
            (about: "Compresses a dataset into a single .bzip archive")
            (@arg DIRECTORY: +required "Path to gtfs directory which should be compressed")
            (@arg ARCHIVE: +required "Path where the zipped archive should be created")
        )
        (@subcommand import =>
            (@arg DATASET: +required "Path to gtfs dataset")
            (@arg PROFILE: --profile +takes_value "Profile used for importing")
            (@arg DATE: --date "Date in the format yyyy-mm-dd")
        )
        (@subcommand load =>
            (@arg DATA: default_value("wasm/www/data.bin") "Path to imported data")
        )
        (@subcommand inspect =>
            (@arg DATASET: +required "Path to gtfs dataset")
            (@arg LINE: "Line name to inspect")
        )
    )
    .get_matches();

    match matches.subcommand() {
        ("compress", Some(compress_matches)) => {
            let directory = compress_matches.value_of_os("DIRECTORY").unwrap();
            let archive = compress_matches.value_of_os("ARCHIVE").unwrap();
            compress(directory, archive)
        }
        ("import", Some(import_matches)) => {
            let dataset = import_matches.value_of_os("DATASET").unwrap();
            let profile = match import_matches.value_of("PROFILE") {
                Some(profile_name) => Profile::try_from(profile_name)?,
                None => Profile::default(),
            };
            let date = match import_matches.value_of("DATE") {
                Some(date) => NaiveDate::parse_from_str(date, "%F")?,
                None => NaiveDate::from_ymd(2019, 8, 26),
            };
            let imported = ImportedDataset::import(dataset)?;
            let file = File::create("wasm/www/data.bin")?;
            imported.store_into(file, profile, date)?;
            Ok(())
        }
        ("load", Some(load_matches)) => {
            let data = load_matches.value_of_os("DATA").unwrap();
            load(data)
        }
        ("inspect", Some(inspect_matches)) => {
            let dataset = inspect_matches.value_of_os("DATASET").unwrap();
            let line_name = inspect_matches.value_of("LINE").unwrap();
            inspect(dataset, line_name)
        }
        ("", None) => Ok(()),
        _ => unreachable!(),
    }
}
