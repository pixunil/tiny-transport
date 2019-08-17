use std::rc::Rc;

use serde_derive::{Serialize, Deserialize};

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

    pub fn unfreeze(self, id: usize) -> Rc<simulation::Station> {
        Rc::new(simulation::Station::new(id, self.position, self.name))
    }
}
