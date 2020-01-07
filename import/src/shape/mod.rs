mod shape;
mod record;
mod importer;

#[cfg(test)]
pub(crate) mod fixtures {
    pub(crate) use super::shape::fixtures as shapes;
}

use record::ShapeRecord;

pub(crate) use shape::{Shape, ShapeId, transform};
pub(crate) use importer::Importer;
