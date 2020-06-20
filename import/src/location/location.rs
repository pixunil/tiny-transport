use std::fmt;

use crate::coord::{debug_position, transform, Point};
use crate::create_id_type;

create_id_type!(LocationId);

#[derive(PartialEq)]
pub struct Location {
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn position(&self) -> Point {
        self.position
    }

    pub(crate) fn store(&self) -> storage::Station {
        let position = transform(self.position());
        storage::Station::new(position, self.name.clone())
    }
}

#[cfg_attr(tarpaulin, skip)]
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
    use std::collections::HashMap;
    use std::rc::Rc;

    use super::*;
    use crate::coord::project;
    use test_utils::map;

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
        bellevue:                            52.520, 13.347, "Bellevue";
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
        clara_jaschke_str:                   52.525, 13.366, "Clara-Jaschke-Str.";
        lueneburger_str:                     52.523, 13.362, "Lüneburger Str.";
        lesser_ury_weg:                      52.524, 13.362, "Lesser-Ury-Weg";
        invalidenpark:                       52.529, 13.377, "Invalidenpark";
        naturkundemuseum:                    52.530, 13.382, "Naturkundemuseum";
        nordbahnhof:                         52.532, 13.389, "Nordbahnhof";
        gedenkstaette_berliner_mauer:        52.536, 13.390, "Gedenkstätte Berliner Mauer";
        bernauer_str:                        52.538, 13.396, "Bernauer Str.";
        wolliner_str:                        52.540, 13.402, "Wolliner Str.";
        friedrich_ludwig_jahn_sportpark:     52.541, 13.406, "Friedrich-Ludwig-Jahn-Sportpark";
        eberswalder_str:                     52.541, 13.412, "Eberswalder Str.";
        husemannstr:                         52.540, 13.419, "Husemannstr.";
        prenzlauer_allee_danziger_str:       52.539, 13.424, "Prenzlauer Allee/Danziger Str.";
        winsstr:                             52.538, 13.429, "Winsstr.";
        greifswalder_str_danziger_str:       52.536, 13.433, "Greifswalder Str./Danziger Str.";
        arnswalder_platz:                    52.534, 13.437, "Arnswalder Platz";
        landsberger_allee_petersburger_str:  52.526, 13.447, "Landsberger Allee/Petersburger Str.";
        strassmannstr:                       52.523, 13.450, "Straßmannstr.";
        bersarinplatz:                       52.519, 13.453, "Bersarinplatz";
        warschauer_str:                      52.506, 13.449, "Warschauer Str.";
        revaler_str:                         52.509, 13.451, "Revaler Str.";
        gruenberger_str_warschauer_str:      52.512, 13.452, "Grünberger Str./Warschauer Str.";
        frankfurter_tor:                     52.516, 13.454, "Frankfurter Tor";
        kniprodestr_danziger_str:            52.532, 13.442, "Kniprodestr./Danziger Str.";
        paul_heyse_str:                      52.529, 13.445, "Paul-Heyse-Str.";
        universitaetsstr:                    52.519, 13.392, "Universitätsstr.";
        am_kupfergraben:                     52.519, 13.395, "Am Kupfergraben";
        georgenstr_am_kupfergraben:          52.520, 13.394, "Georgenstr./Am Kupfergraben";
        anhalter_bahnhof:                    52.505, 13.382, "Anhalter Bahnhof";
        abgeordnetenhaus:                    52.507, 13.380, "Abgeordnetenhaus";
        potsdamer_platz_bus_stresemannstr:   52.509, 13.377, "Potsdamer Platz [Bus Stresemannstr.]";
        potsdamer_platz_vossstr:             52.510, 13.377, "Potsdamer Platz/Voßstr.";
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
    use crate::fixtures::locations;

    #[test]
    fn test_getters() {
        let location = locations::hauptbahnhof();
        assert_eq!(location.id(), "hauptbahnhof".into());
        assert_eq!(location.position(), project(52.526, 13.369));
    }

    #[test]
    fn test_store() {
        let location = locations::hauptbahnhof();
        assert_eq!(
            location.store(),
            storage::fixtures::stations::hauptbahnhof()
        );
    }
}
