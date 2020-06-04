use std::convert::TryInto;
use std::error::Error;
use std::fs::File;

use chrono::NaiveDate;
use clap::{clap_app, value_t};

use import::profile::{DEFAULT_PROFILE_NAME, PROFILE_NAMES};
use import::ImportedDataset;

mod compress;
mod inspect;
mod load;

use compress::compress;
use inspect::{inspect, Format};
use load::load;

fn validate_date(value: String) -> Result<(), String> {
    NaiveDate::parse_from_str(&value, "%F")
        .map(|_| ())
        .map_err(|error| format!("{}, it must be in the format yyyy-mm-dd", error))
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app!(gtfs_sim =>
        (@subcommand compress =>
            (about: "Compresses a dataset into a single .bzip archive")
            (@arg DIRECTORY: +required "Path to gtfs directory which should be compressed")
            (@arg ARCHIVE: +required "Path where the zipped archive should be created")
        )
        (@subcommand import =>
            (about: "Imports a dataset and generates a binary export")
            (@arg DATASET: +required "Path to gtfs dataset")
            (@arg PROFILE: --profile +takes_value
                possible_values(PROFILE_NAMES) default_value(DEFAULT_PROFILE_NAME)
                "Profile used for importing")
            (@arg DATE: --date +takes_value {validate_date} default_value("2019-08-26")
                "Date in the format yyyy-mm-dd")
        )
        (@subcommand load =>
            (about: "Loads a binary export to check for possible errors")
            (@arg DATA: default_value("wasm/www/data.bin") "Path to imported data")
        )
        (@subcommand inspect =>
            (about: "Imports a dataset and prints debug information for a single line")
            (@arg DATASET: +required "Path to gtfs dataset")
            (@arg LINE: +required "Line name to inspect")
            (@arg FORMAT: --format +takes_value +case_insensitive
                possible_values(&Format::variants()) default_value("import")
                "Output format")
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
            let profile = import_matches.value_of("PROFILE").unwrap().try_into()?;
            let date_format = import_matches.value_of("DATE").unwrap();
            let date = NaiveDate::parse_from_str(date_format, "%F")?;
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
            let format = value_t!(inspect_matches, "FORMAT", Format).unwrap_or_else(|e| e.exit());
            inspect(dataset, line_name, format)
        }
        ("", None) => Ok(()),
        _ => unreachable!(),
    }
}
