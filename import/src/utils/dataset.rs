use std::error::Error;
use std::io::{Read, Seek};
use std::path::PathBuf;
use std::fs::File;

use csv::Reader as Csv;

use zip::read::ZipArchive;

pub trait Dataset {
    fn read_csv<'a>(&'a mut self, name: &str) -> Result<csv::Reader<Box<dyn Read + 'a>>, Box<dyn Error>>;
}

impl Dataset for PathBuf {
    fn read_csv<'a>(&'a mut self, name: &str) -> Result<csv::Reader<Box<dyn Read + 'a>>, Box<dyn Error>> {
        self.set_file_name(name);
        let file = File::open(&self)?;
        Ok(Csv::from_reader(Box::new(file)))
    }
}

impl<R: Read + Seek> Dataset for ZipArchive<R> {
    fn read_csv<'a>(&'a mut self, name: &str) -> Result<csv::Reader<Box<dyn Read + 'a>>, Box<dyn Error>> {
        let file = self.by_name(name)?;
        Ok(Csv::from_reader(Box::new(file)))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::io;
    use std::collections::HashMap;

    impl Dataset for HashMap<String, String> {
        fn read_csv<'a>(&'a mut self, name: &str) -> Result<Csv<Box<dyn Read + 'a>>, Box<dyn Error>> {
            let data = self.get(name).ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
            Ok(Csv::from_reader(Box::new(data.as_bytes())))
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
        })
    }
}
