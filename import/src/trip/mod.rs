mod importer;
mod node;
mod record;
mod route;
mod route_buffer;
mod route_variant;
mod scheduler;
mod trip;
mod trip_buffer;

#[cfg(test)]
pub(crate) mod fixtures {
    pub(crate) use super::node::fixtures as nodes;
    pub(crate) use super::route::fixtures as routes;
    pub(crate) use super::route_buffer::fixtures as route_buffers;
    pub(crate) use super::route_variant::fixtures as route_variants;
    pub(crate) use super::trip::fixtures as trips;
    pub(crate) use super::trip_buffer::fixtures as trip_buffers;
}

use node::Node;
use record::{StopRecord, TripRecord};
use route_buffer::RouteBuffer;
use route_variant::RouteVariant;
use scheduler::Scheduler;
use trip::Trip;
use trip_buffer::{TripBuffer, TripId};

pub(crate) use importer::Importer;
pub use route::Route;
