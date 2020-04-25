use std::cmp::Ordering;
use std::fmt;

use crate::coord::{debug_position, transform, Point};
use crate::create_id_type;

create_id_type!(LocationId);

#[derive(PartialEq)]
pub(crate) struct Location {
    id: LocationId,
    name: String,
    position: Point,
}

impl Location {
    pub(crate) fn new(id: LocationId, name: String, position: Point) -> Location {
        Location { id, name, position }
    }

    pub(crate) fn id(&self) -> LocationId {
        self.id.clone()
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

impl fmt::Debug for Location {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let position = debug_position(self.position, formatter.alternate());
        formatter
            .debug_struct("Location")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("position", &position)
            .finish()
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;
    use crate::coord::project;
    use crate::map;
    use std::collections::HashMap;
    use std::rc::Rc;

    macro_rules! locations {
        ($($location:ident: $lat:expr, $lon:expr, $name:expr);* $(;)?) => (
            $(
                pub(crate) fn $location() -> Location {
                    Location::new(stringify!($location).into(), $name.to_string(), project($lat, $lon))
                }
            )*

            pub(crate) fn by_id() -> HashMap<LocationId, Rc<Location>> {
                map! {
                    $( stringify!($location) => Rc::new($location()) ),*
                }
            }
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
        nollendorfplatz:                     52.500, 13.354, "Nollendorfplatz";
        viktoria_luise_platz:                52.496, 13.343, "Viktoria-Luise-Platz";
        bayerischer_platz:                   52.489, 13.340, "Bayerischer Platz";
        rathaus_schoeneberg:                 52.483, 13.342, "Rathaus Schöneberg";
        innsbrucker_platz:                   52.478, 13.343, "Innsbrucker Platz";
        oranienburger_tor:                   52.525, 13.388, "Oranienburger Tor";
        universitaetsstr:                    52.519, 13.392, "Universitätsstr.";
        am_kupfergraben:                     52.519, 13.395, "Am Kupfergraben";
        georgenstr_am_kupfergraben:          52.520, 13.394, "Georgenstr./Am Kupfergraben";
        wannsee:                             52.421, 13.179, "Wannsee";
        wannseebruecke:                      52.420, 13.175, "Wannseebrücke";
        am_kleinen_wannsee:                  52.420, 13.167, "Am Kleinen Wannsee";
        seglerweg:                           52.424, 13.161, "Seglerweg";
        koblanckstr:                         52.427, 13.162, "Koblanckstr.";
        liebermann_villa:                    52.429, 13.164, "Liebermann-Villa";
        am_grossen_wannsee:                  52.432, 13.165, "Am Großen Wannsee";
        haus_der_wannsee_konferenz:          52.433, 13.164, "Haus der Wannsee-Konferenz";
        zum_heckeshorn:                      52.430, 13.161, "Zum Heckeshorn";
        strasse_zum_loewen:                  52.427, 13.160, "Straße zum Löwen";
        conradstr:                           52.420, 13.162, "Conradstr.";
    }
}

#[cfg(test)]
mod tests {
    use crate::coord::project;
    use crate::location::fixtures::*;

    #[test]
    fn test_getters() {
        let location = locations::hauptbahnhof();
        assert_eq!(location.id(), "hauptbahnhof".into());
        assert_eq!(location.position(), project(52.526, 13.369));
    }
}
