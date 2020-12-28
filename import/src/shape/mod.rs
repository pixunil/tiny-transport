mod buffer;
mod importer;
mod record;
mod segment;
mod segmenter;
mod shape;
mod shapes;
mod smoother;

#[cfg(test)]
pub(crate) mod fixtures {
    pub(crate) use super::buffer::fixtures as shape_buffers;
    pub(crate) use super::segment::fixtures as segments;
    pub(crate) use super::shape::fixtures as shapes;
}

use buffer::Buffer;
use record::ShapeRecord;
use segmenter::Segmenter;

pub(crate) use buffer::ShapeId;
pub(crate) use importer::Importer;
pub(crate) use segment::Segment;
pub(crate) use shape::Order;
pub(crate) use shape::PointRef;
pub(crate) use shape::SegmentRef;
pub(crate) use shape::SegmentedShape;
pub(crate) use shapes::Shapes;
pub use smoother::Mode as SmoothMode;
