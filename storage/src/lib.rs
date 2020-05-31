use serde_derive::{Deserialize, Serialize};

mod line;
mod station;
mod train;

pub use crate::line::Line;
pub use crate::station::Station;
pub use crate::train::Train;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dataset {
    pub stations: Vec<Station>,
    pub lines: Vec<Line>,
}

impl Dataset {
    pub fn new(stations: Vec<Station>, lines: Vec<Line>) -> Self {
        Self { stations, lines }
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    pub use crate::line::fixtures as lines;
    pub use crate::station::fixtures as stations;
    pub use crate::train::fixtures as trains;
}
