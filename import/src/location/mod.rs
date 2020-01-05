mod kind;
mod location;
mod errors;
mod record;
mod importer;

#[cfg(test)]
pub(crate) mod fixtures {
    pub(crate) use super::location::fixtures as locations;
}

use kind::LocationKind;
use errors::LocationImportError;
use record::LocationRecord;

pub(crate) use location::{Location, LocationId};
pub(crate) use importer::Importer;
