use serde_derive::{Deserialize, Serialize};

mod line;
mod station;
mod train;

pub use crate::line::{Line, LineGroup};
pub use crate::station::Station;
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

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    pub use crate::line::fixtures as lines;
    pub use crate::station::fixtures as stations;
    pub use crate::train::fixtures as trains;
}
