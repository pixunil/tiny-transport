use std::error::Error;
use std::io::{Read, Seek};
use std::path::PathBuf;
use std::fs::File;

use zip::read::ZipArchive;

pub trait Dataset {
    fn read_csv<'a>(&'a mut self, name: &str) -> Result<csv::Reader<Box<dyn Read + 'a>>, Box<dyn Error>>;
}

impl Dataset for PathBuf {
    fn read_csv<'a>(&'a mut self, name: &str) -> Result<csv::Reader<Box<dyn Read + 'a>>, Box<dyn Error>> {
        self.set_file_name(name);
        let file = File::open(&self)?;
        Ok(csv::Reader::from_reader(Box::new(file)))
    }
}

impl<R: Read + Seek> Dataset for ZipArchive<R> {
    fn read_csv<'a>(&'a mut self, name: &str) -> Result<csv::Reader<Box<dyn Read + 'a>>, Box<dyn Error>> {
        let file = self.by_name(name)?;
        Ok(csv::Reader::from_reader(Box::new(file)))
    }
}
