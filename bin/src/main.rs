use std::convert::TryInto;
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

use chrono::NaiveDate;
use clap::{clap_app, value_t};
use rustyline::Editor;

use import::profile::{DEFAULT_PROFILE_NAME, PROFILE_NAMES};
use import::shape::SmoothMode;
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

#[derive(Debug)]
struct NoDatasetImportedError;

impl fmt::Display for NoDatasetImportedError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "no dataset imported")
    }
}

impl Error for NoDatasetImportedError {}

struct CommandRunner {
    dataset: Option<ImportedDataset>,
}

impl CommandRunner {
    fn new() -> Self {
        Self { dataset: None }
    }

    fn build_app(&self, interactive: bool) -> clap::App<'static, 'static> {
        let app = clap_app!(gtfs_sim =>
            (@subcommand compress =>
                (about: "Compresses a dataset into a single .bzip archive")
                (@arg directory: <DIRECTORY> "Path to gtfs directory which should be compressed")
                (@arg archive: <ARCHIVE> "Path where the zipped archive should be created")
            )
            (@subcommand import =>
                (about: "Imports a dataset")
                (@setting TrailingVarArg)
                (@arg dataset: <DATASET> "Path to gtfs dataset")
                (@arg shape_smoothing: --("shape-smoothing") [MODE] +case_insensitive
                    possible_values(&SmoothMode::variants()) default_value("full")
                    "Smooth mode for processing shapes")
                (@arg command: [COMMAND] +last +multiple "Command to run afterwards"))
            (@subcommand inspect =>
                (about: "Inspects the imported dataset")
                (@arg agency_name: --agency [AGENCY] "Filter after agency name")
                (@arg line_name: <LINE> "Line name to inspect")
                (@arg format: --format [FORMAT] +case_insensitive
                    possible_values(&Format::variants()) default_value("import")
                    "Output format")
                (@arg output: --output [FILE] "Path to output file")
            )
            (@subcommand store =>
                (about: "Generates a binary export from the imported dataset")
                (@arg profile: --profile [PROFILE]
                    possible_values(PROFILE_NAMES) default_value(DEFAULT_PROFILE_NAME)
                    "Profile used for exporting")
                (@arg date: --date [DATE] {validate_date} default_value("2019-08-26")
                    "Date in the format yyyy-mm-dd"))
            (@subcommand load =>
                (about: "Loads a binary export to check for possible errors")
                (@arg binary: [BINARY] default_value("wasm/www/data.bin") "Path to stored data")));

        if interactive {
            app.setting(clap::AppSettings::NoBinaryName)
                .setting(clap::AppSettings::DisableVersion)
        } else {
            app
        }
    }

    fn history_path() -> PathBuf {
        let mut path = dirs::data_dir().unwrap();
        path.push("gtfs-sim");
        fs::create_dir_all(&path).unwrap();
        path.push("history.txt");
        path
    }

    fn dataset(&self) -> Result<&ImportedDataset, NoDatasetImportedError> {
        self.dataset.as_ref().ok_or(NoDatasetImportedError)
    }

    fn run_with_program_arguments(&mut self) {
        let app = self.build_app(false);
        let matches = app.get_matches();
        let should_run_interactively = self.execute_or_print(matches);
        if should_run_interactively {
            self.run_interactively();
        }
    }

    fn run_interactively(&mut self) {
        let mut editor = Editor::<()>::new();
        let _ = editor.load_history(&Self::history_path());
        loop {
            let app = self.build_app(true);
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
                    self.execute_or_print(matches);
                }
                Err(error) => {
                    println!("{}", error);
                }
            }
        }
        let _ = editor.save_history(&Self::history_path());
    }

    fn execute(&mut self, matches: clap::ArgMatches) -> Result<bool, Box<dyn Error>> {
        match matches.subcommand() {
            ("compress", Some(compress_matches)) => {
                let directory = compress_matches.value_of_os("directory").unwrap();
                let archive = compress_matches.value_of_os("archive").unwrap();
                compress(directory, archive)?;
            }
            ("import", Some(import_matches)) => {
                let command_matches = import_matches.values_of("command").map(|command| {
                    let app = self.build_app(true);
                    app.get_matches_from(command)
                });
                let path = import_matches.value_of_os("dataset").unwrap();
                let shape_smoothing = value_t!(import_matches, "shape_smoothing", SmoothMode)?;
                self.dataset = Some(ImportedDataset::import(path, shape_smoothing)?);
                if let Some(command_matches) = command_matches {
                    return self.execute(command_matches);
                }
                return Ok(true);
            }
            ("store", Some(store_matches)) => {
                let profile = store_matches.value_of("profile").unwrap().try_into()?;
                let date_formatted = store_matches.value_of("date").unwrap();
                let date = NaiveDate::parse_from_str(date_formatted, "%F")?;
                let file = File::create("wasm/www/data.bin")?;
                self.dataset()?.store_into(file, profile, date)?;
            }
            ("load", Some(load_matches)) => {
                let binary = load_matches.value_of_os("binary").unwrap();
                load(binary)?;
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
                inspect(self.dataset()?, agency_name, line_name, format, &mut output)?;
            }
            ("", None) => return Ok(true),
            _ => unreachable!(),
        }
        Ok(false)
    }

    fn execute_or_print(&mut self, matches: clap::ArgMatches) -> bool {
        self.execute(matches).unwrap_or_else(|error| {
            println!("{}", error);
            true
        })
    }
}

fn main() {
    CommandRunner::new().run_with_program_arguments();
}
