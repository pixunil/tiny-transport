use std::error::Error;
use std::fmt;

use super::LocationRecord;

#[derive(Debug)]
pub(super) enum LocationImportError {
    StationHasParent(LocationRecord),
    ParentNotFound(LocationRecord),
}

impl fmt::Display for LocationImportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocationImportError::StationHasParent(record) => write!(
                formatter,
                "forbidden parent {} for station {}",
                record.parent_station().unwrap(),
                record.stop_id()
            ),
            LocationImportError::ParentNotFound(record) => write!(
                formatter,
                "parent {} for location {} not found",
                record.parent_station().unwrap(),
                record.stop_id()
            ),
        }
    }
}

impl Error for LocationImportError {}
