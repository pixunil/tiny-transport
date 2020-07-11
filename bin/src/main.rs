use std::convert::TryInto;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Write};
use std::ops::Deref;
use std::path::PathBuf;

use chrono::NaiveDate;
use clap::{clap_app, value_t};
use rustyline::Editor;

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

enum ImportedDatasetRef<'a> {
    Temporary(ImportedDataset),
    Stored(&'a ImportedDataset),
}

impl<'a> Deref for ImportedDatasetRef<'a> {
    type Target = ImportedDataset;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Temporary(dataset) => &dataset,
            Self::Stored(dataset) => dataset,
        }
    }
}

enum CommandRunner {
    ProgramArguments,
    Interactive { dataset: Option<ImportedDataset> },
}

impl CommandRunner {
    fn program_arguments() -> Self {
        Self::ProgramArguments
    }

    fn interactive(dataset: Option<ImportedDataset>) -> Self {
        Self::Interactive { dataset }
    }

    fn build_app(&self) -> clap::App<'static, 'static> {
        let is_dataset_available = match self {
            Self::ProgramArguments => false,
            Self::Interactive { dataset } => dataset.is_some(),
        };
        let app = clap_app!(gtfs_sim =>
            (@subcommand compress =>
                (about: "Compresses a dataset into a single .bzip archive")
                (@arg directory: <DIRECTORY> "Path to gtfs directory which should be compressed")
                (@arg archive: <ARCHIVE> "Path where the zipped archive should be created")
            )
            (@subcommand import =>
                (about: "Imports a dataset")
                (@arg dataset: <DATASET> "Path to gtfs dataset"))
            (@subcommand inspect =>
                (about: "Inspects the dataset")
                (@arg dataset: --dataset [DATASET] required(!is_dataset_available)
                    "Path to gtfs dataset")
                (@arg agency_name: --agency [AGENCY] "Filter after agency name")
                (@arg line_name: <LINE> "Line name to inspect")
                (@arg format: --format [FORMAT] +case_insensitive
                    possible_values(&Format::variants()) default_value("import")
                    "Output format")
                (@arg output: --output [FILE] "Path to output file")
            )
            (@subcommand store =>
                (about: "Generates a binary export")
                (@arg dataset: --dataset [DATASET] required(!is_dataset_available)
                    "Path to gtfs dataset")
                (@arg profile: --profile [PROFILE]
                    possible_values(PROFILE_NAMES) default_value(DEFAULT_PROFILE_NAME)
                    "Profile used for exporting")
                (@arg date: --date [DATE] {validate_date} default_value("2019-08-26")
                    "Date in the format yyyy-mm-dd"))
            (@subcommand load =>
                (about: "Loads a binary export to check for possible errors")
                (@arg binary: [BINARY] default_value("wasm/www/data.bin") "Path to stored data")));

        match self {
            Self::Interactive { .. } => app
                .setting(clap::AppSettings::NoBinaryName)
                .setting(clap::AppSettings::DisableVersion),
            _ => app,
        }
    }

    fn history_path() -> PathBuf {
        let mut path = dirs::data_dir().unwrap();
        path.push("gtfs-sim");
        fs::create_dir_all(&path).unwrap();
        path.push("history.txt");
        path
    }

    fn retrieve_dataset(
        &self,
        matches: &clap::ArgMatches,
    ) -> Result<ImportedDatasetRef, Box<dyn Error>> {
        if let Some(path) = matches.value_of_os("dataset") {
            let dataset = ImportedDataset::import(path)?;
            Ok(ImportedDatasetRef::Temporary(dataset))
        } else {
            match self {
                Self::Interactive {
                    dataset: Some(dataset),
                } => Ok(ImportedDatasetRef::Stored(&dataset)),
                _ => unreachable!(),
            }
        }
    }

    fn run(&mut self) {
        match self {
            Self::ProgramArguments => {
                let app = self.build_app();
                let matches = app.get_matches();
                self.execute(matches)
                    .unwrap_or_else(|error| println!("{}", error));
            }
            Self::Interactive { .. } => {
                let mut editor = Editor::<()>::new();
                let _ = editor.load_history(&Self::history_path());
                loop {
                    let app = self.build_app();
                    let line = match editor.readline("> ") {
                        Ok(line) => line,
                        Err(_) => break,
                    };
                    let arguments = match shlex::split(&line) {
                        Some(arguments) => arguments,
                        None => {
                            println!("Erroneous input");
                            continue;
                        }
                    };
                    let matches = app.get_matches_from_safe(arguments);
                    match matches {
                        Ok(matches) => {
                            editor.add_history_entry(line);
                            self.execute(matches)
                                .unwrap_or_else(|error| println!("{}", error));
                        }
                        Err(error) => {
                            println!("{}", error);
                        }
                    }
                }
                let _ = editor.save_history(&Self::history_path());
            }
        }
    }

    fn execute(&mut self, matches: clap::ArgMatches) -> Result<(), Box<dyn Error>> {
        match matches.subcommand() {
            ("compress", Some(compress_matches)) => {
                let directory = compress_matches.value_of_os("directory").unwrap();
                let archive = compress_matches.value_of_os("archive").unwrap();
                compress(directory, archive)
            }
            ("import", Some(import_matches)) => {
                let path = import_matches.value_of_os("dataset").unwrap();
                let imported_dataset = ImportedDataset::import(path)?;
                match self {
                    Self::ProgramArguments => {
                        *self = Self::interactive(Some(imported_dataset));
                        self.run();
                    }
                    Self::Interactive {
                        ref mut dataset, ..
                    } => {
                        *dataset = Some(imported_dataset);
                    }
                }
                Ok(())
            }
            ("store", Some(store_matches)) => {
                let profile = store_matches.value_of("profile").unwrap().try_into()?;
                let date_formatted = store_matches.value_of("date").unwrap();
                let date = NaiveDate::parse_from_str(date_formatted, "%F")?;
                let dataset = self.retrieve_dataset(store_matches)?;

                let file = File::create("wasm/www/data.bin")?;
                dataset.store_into(file, profile, date)?;
                Ok(())
            }
            ("load", Some(load_matches)) => {
                let binary = load_matches.value_of_os("binary").unwrap();
                load(binary)
            }
            ("inspect", Some(inspect_matches)) => {
                let line_name = inspect_matches.value_of("line_name").unwrap();
                let agency_name = inspect_matches.value_of("agency_name");
                let format =
                    value_t!(inspect_matches, "format", Format).unwrap_or_else(|e| e.exit());
                let mut output: Box<dyn Write> = match inspect_matches.value_of_os("output") {
                    Some(path) => Box::new(File::create(path)?),
                    None => Box::new(io::stdout()),
                };
                let dataset = self.retrieve_dataset(&inspect_matches)?;
                inspect(&dataset, agency_name, line_name, format, &mut output)
            }
            ("", None) => {
                match self {
                    Self::ProgramArguments => {
                        *self = Self::interactive(None);
                        self.run();
                    }
                    Self::Interactive { .. } => {}
                }
                Ok(())
            }
            _ => unreachable!(),
        }
    }
}

fn main() {
    CommandRunner::program_arguments().run();
}
