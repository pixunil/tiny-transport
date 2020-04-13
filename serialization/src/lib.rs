use serde_derive::{Serialize, Deserialize};

mod station;
mod line;
pub mod train;

pub use crate::station::Station;
pub use crate::line::{Line, LineGroup};
pub use crate::train::Train;

#[derive(Debug, Serialize, Deserialize)]
pub struct Dataset {
    pub stations: Vec<Station>,
    pub line_groups: Vec<LineGroup>,
}

impl Dataset {
    pub fn new(stations: Vec<Station>, line_groups: Vec<LineGroup>) -> Dataset {
        Dataset {
            stations,
            line_groups,
        }
    }
}
