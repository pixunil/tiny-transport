pub mod deserialize;
mod dataset;

pub use serde_derive::Deserialize;

pub use dataset::Dataset;

pub type Id = String;
