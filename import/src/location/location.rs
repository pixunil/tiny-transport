use std::cmp::Ordering;

use crate::create_id_type;
use crate::coord::{Point, transform};

create_id_type!(LocationId);

#[derive(Debug, PartialEq)]
pub(crate) struct Location {
    pub id: LocationId,
    pub name: String,
    position: Point,
}

impl Location {
    pub(crate) fn new(id: LocationId, name: String, position: Point) -> Location {
        Location { id, name, position }
    }

    pub(crate) fn position(&self) -> Point {
        self.position
    }

    pub(crate) fn station_cmp(&self, other: &Location) -> Ordering {
        self.id.cmp(&other.id)
    }

    pub(crate) fn freeze(&self) -> serialization::Station {
        let position = transform(self.position());
        serialization::Station::new(position, self.name.clone())
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;
    use crate::coord::project;

    macro_rules! locations {
        ($($location:ident: $lat:expr, $lon:expr, $name:expr);* $(;)?) => (
            $(
                pub(crate) fn $location() -> Location {
                    Location::new(stringify!($location).into(), $name.to_string(), project($lat, $lon))
                }
            )*
         );
    }

    locations! {
        hauptbahnhof:                        52.526, 13.369, "Hauptbahnhof";
        friedrichstr:                        52.520, 13.387, "Friedrichstr.";
        hackescher_markt:                    52.523, 13.402, "Hackescher Markt";
        gesundbrunnen:                       52.549, 13.388, "Gesundbrunnen";
        ostkreuz:                            52.503, 13.469, "Ostkreuz";
        suedkreuz:                           52.475, 13.366, "Südkreuz";
        westkreuz:                           52.501, 13.283, "Westkreuz";
        oranienburger_tor:                   52.525, 13.388, "Oranienburger Tor";
        universitaetsstr:                    52.519, 13.392, "Universitätsstr.";
        am_kupfergraben:                     52.519, 13.395, "Am Kupfergraben";
        georgenstr_am_kupfergraben:          52.520, 13.394, "Georgenstr./Am Kupfergraben";
    }
}
