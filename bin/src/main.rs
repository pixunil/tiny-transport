use std::convert::TryFrom;
use std::error::Error;

use clap::clap_app;

use import::profile::Profile;
use import::import;

mod compress;
mod load;

use compress::compress;
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
        )
        (@subcommand load =>
            (@arg DATA: default_value("wasm/www/data.bin") "Path to imported data")
        )
    ).get_matches();

    match matches.subcommand() {
        ("compress", Some(compress_matches)) => {
            let directory = compress_matches.value_of_os("DIRECTORY").unwrap();
            let archive = compress_matches.value_of_os("ARCHIVE").unwrap();
            compress(directory, archive)
        },
        ("import", Some(import_matches)) => {
            let dataset = import_matches.value_of_os("DATASET").unwrap();
            let profile = match import_matches.value_of("PROFILE") {
                Some(profile_name) => Profile::try_from(profile_name)?,
                None => Profile::default(),
            };
            import(dataset, profile)
        },
        ("load", Some(load_matches)) => {
            let data = load_matches.value_of_os("DATA").unwrap();
            load(data)
        },
        ("", None) => Ok(()),
        _ => unreachable!(),
    }
}
