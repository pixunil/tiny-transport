use std::error::Error;

use clap::clap_app;

use import::import;

mod compress;

use compress::compress;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app!(gtfs_sim =>
        (@subcommand compress =>
            (about: "Compresses a dataset into a single .bzip archive")
            (@arg DIRECTORY: +required "Path to gtfs directory which should be compressed")
            (@arg ARCHIVE: +required "Path where the zipped archive should be created")
        )
        (@subcommand import =>
            (@arg DATASET: +required "Path to gtfs dataset")
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
            import(dataset)
        },
        ("", None) => Ok(()),
        _ => unreachable!(),
    }
}
