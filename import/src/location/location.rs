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

#[cfg(test)]
mod tests {
    #[macro_export]
    macro_rules! station {
        ($id:expr, $name:expr, $lat:expr, $lon:expr) => (
            $crate::location::Location::new($id.into(), $name.to_string(), ::na::Point2::new($lon, $lat))
        );
        (main_station) => (
            $crate::station!("1", "Main Station", 52.526, 13.369)
        );
        (center) => (
            $crate::station!("2", "Center", 52.520, 13.387)
        );
        (market) => (
            $crate::station!("3", "Market", 52.523, 13.402)
        );
        (north_cross) => (
            $crate::station!("4", "North Cross", 52.549, 13.388)
        );
        (east_cross) => (
            $crate::station!("5", "East Cross", 52.503, 13.469)
        );
        (south_cross) => (
            $crate::station!("6", "South Cross", 52.475, 13.366)
        );
        (west_cross) => (
            $crate::station!("7", "West Cross", 52.501, 13.283)
        );
        ($($station:ident),*) => (
            vec![$(Rc::new($crate::station!($station))),*]
        );
    }
}
