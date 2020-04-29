use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Seek};
use std::path::PathBuf;

use zip::read::ZipArchive;

pub(crate) struct Table<'r> {
    #[cfg_attr(not(feature = "progress"), allow(dead_code))]
    pub(super) size: u64,
    pub(super) reader: Box<dyn Read + 'r>,
}

impl<'r> Table<'r> {
    fn new<R: Read + 'r>(size: u64, reader: R) -> Self {
        Self {
            reader: Box::new(reader),
            size,
        }
    }
}

pub(crate) trait Dataset {
    type Error: Error + 'static;

    fn open_csv(&mut self, name: &str) -> Result<Table, Self::Error>;
}

impl Dataset for PathBuf {
    type Error = io::Error;

    fn open_csv(&mut self, name: &str) -> Result<Table, Self::Error> {
        self.set_file_name(name);
        let file = File::open(&self)?;
        Ok(Table::new(file.metadata().unwrap().len(), file))
    }
}

impl<R: Read + Seek> Dataset for ZipArchive<R> {
    type Error = zip::result::ZipError;

    fn open_csv(&mut self, name: &str) -> Result<Table, Self::Error> {
        let file = self.by_name(name)?;
        Ok(Table::new(file.size(), file))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io;

    use super::*;

    impl Dataset for HashMap<String, String> {
        type Error = io::Error;

        fn open_csv(&mut self, name: &str) -> Result<Table, Self::Error> {
            let data = self
                .get(name)
                .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
            Ok(Table::new(data.len() as u64, data.as_bytes()))
        }
    }

    #[macro_export]
    macro_rules! dataset {
        ($($name:ident: $($($value:expr),+);+)*) => ({
            let mut dataset = HashMap::new();
            $(
                let mut data = String::new();
                $(
                    $(
                        data += stringify!($value);
                        data += ",";
                    )*
                    data += "\n";
                )+
                dataset.insert(format!("{}.txt", stringify!($name)), data);
            )*
            dataset
        });
    }
}
