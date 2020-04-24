use serde_derive::{Deserialize, Serialize};

use na::Point2;

#[derive(Debug, Serialize, Deserialize)]
pub struct Station {
    position: Point2<f32>,
    name: String,
}

impl Station {
    pub fn new(position: Point2<f32>, name: String) -> Station {
        Station { position, name }
    }

    pub fn unfreeze(self) -> simulation::Station {
        simulation::Station::new(self.position, self.name)
    }
}
