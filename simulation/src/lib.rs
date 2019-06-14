#[macro_use]
extern crate serde_derive;
extern crate nalgebra as na;

mod color;
mod station;
mod line;
mod track;
mod train;

pub use crate::color::Color;
pub use crate::station::Station;
pub use crate::line::{LineGroup, Line, IndexedLineGroup, IndexedLine};
pub use crate::track::{Connection, TrackBundle};
pub use crate::train::{Train, Direction};

#[derive(Debug, Serialize, Deserialize)]
pub struct Dataset {
    pub stations: Vec<Station>,
    pub line_groups: Vec<IndexedLineGroup>,
}

impl Dataset {
    pub fn new(stations: Vec<Station>, line_groups: Vec<IndexedLineGroup>) -> Dataset {
        Dataset {
            stations,
            line_groups,
        }
    }
}
