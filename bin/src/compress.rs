use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

pub(crate) fn compress(
    directory: impl AsRef<Path>,
    archive: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    let mut zip = ZipWriter::new(File::create(archive)?);
    let options = FileOptions::default().compression_method(CompressionMethod::Bzip2);

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        zip.start_file_from_path(entry.file_name().as_ref(), options)?;

        let data = fs::read(entry.path())?;
        zip.write_all(&data)?;
    }

    zip.finish()?;
    Ok(())
}
