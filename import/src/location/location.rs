use std::cmp::Ordering;

use na::Point2;

use crate::create_id_type;

create_id_type!(LocationId);

#[derive(Debug, PartialEq)]
pub(crate) struct Location {
    pub id: LocationId,
    pub name: String,
    position: Point2<f32>,
}

impl Location {
    pub(crate) fn new(id: LocationId, name: String, position: Point2<f32>) -> Location {
        Location { id, name, position }
    }

    pub(crate) fn position(&self) -> Point2<f32> {
        let x = 2000.0 * (self.position.x - 13.5);
        let y = -4000.0 * (self.position.y - 52.52);
        Point2::new(x, y)
    }

    pub(crate) fn station_cmp(&self, other: &Location) -> Ordering {
        self.id.cmp(&other.id)
    }

    pub(crate) fn freeze(&self) -> serialization::Station {
        serialization::Station::new(self.position(), self.name.clone())
    }
}
