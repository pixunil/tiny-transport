mod errors;
mod importer;
mod kind;
mod linearizer;
mod location;
mod record;

#[cfg(test)]
pub(crate) mod fixtures {
    pub(crate) use super::location::fixtures as locations;
}

use errors::LocationImportError;
use kind::LocationKind;
use record::LocationRecord;

pub(crate) use importer::Importer;
pub(crate) use linearizer::Linearizer;
pub(crate) use location::{Location, LocationId};
