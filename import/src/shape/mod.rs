mod importer;
mod record;
mod shape;

#[cfg(test)]
pub(crate) mod fixtures {
    pub(crate) use super::shape::fixtures as shapes;
}

use record::ShapeRecord;

pub(crate) use importer::Importer;
pub(crate) use shape::{Shape, ShapeId};
