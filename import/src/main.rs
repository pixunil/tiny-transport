extern crate gtfs_sim_import;

use std::env;
use std::error::Error;

use crate::gtfs_sim_import::{compress, import};

fn main() -> Result<(), Box<dyn Error>> {
    let command = env::args().nth(1).unwrap();
    match command.as_str() {
        "compress" => compress(),
        "import" => {
            let path = env::args_os().nth(2).unwrap();
            import(path)
        }
        _ => Ok(()),
    }
}
