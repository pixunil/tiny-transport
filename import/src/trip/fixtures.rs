pub(crate) use crate::location::fixtures::*;
pub(crate) use crate::service::fixtures::*;
pub(crate) use crate::shape::fixtures::*;

pub(super) use super::node::fixtures as nodes;
pub(super) use super::route_buffer::fixtures as route_buffers;
pub(super) use super::route_variant::fixtures as route_variants;
pub(super) use super::trip::fixtures as trips;
pub(super) use super::trip_buffer::fixtures as trip_buffers;

macro_rules! stop_locations {
    ($($line:ident: {$($route:ident => [$($location:ident),* $(,)?]),* $(,)?}),* $(,)?) => (
        pub(crate) mod stop_locations {
            $(
                pub(crate) mod $line {
                    use std::rc::Rc;
                    use crate::location::{Location, fixtures::*};

                    $(
                        pub(crate) fn $route() -> Vec<Rc<Location>> {
                            vec![$(Rc::new(locations::$location())),*]
                        }
                    )*
                }
            )*
        }
    );
}

stop_locations! {
    u4: {
        empty => [],
        nollendorfplatz_innsbrucker_platz => [
            nollendorfplatz, viktoria_luise_platz, bayerischer_platz,
            rathaus_schoeneberg, innsbrucker_platz,
        ],
        innsbrucker_platz_nollendorfplatz => [
            innsbrucker_platz, rathaus_schoeneberg, bayerischer_platz,
            viktoria_luise_platz, nollendorfplatz,
        ],
    },
    tram_12: {
        oranienburger_tor_am_kupfergraben => [
            oranienburger_tor, friedrichstr, universitaetsstr, am_kupfergraben,
        ],
        am_kupfergraben_oranienburger_tor => [
            am_kupfergraben, georgenstr_am_kupfergraben, friedrichstr, oranienburger_tor,
        ],
    },
    bus_m41: {
        anhalter_bahnhof_hauptbahnhof => [
            anhalter_bahnhof, abgeordnetenhaus, potsdamer_platz_bus_stresemannstr,
            potsdamer_platz_vossstr, hauptbahnhof, hauptbahnhof,
        ],
        hauptbahnhof_anhalter_bahnhof => [
            hauptbahnhof, hauptbahnhof, potsdamer_platz_vossstr, potsdamer_platz_bus_stresemannstr,
            abgeordnetenhaus, anhalter_bahnhof,
        ],
    },
    bus_114: {
        wannsee_heckeshorn_wannsee => [
            wannsee, wannseebruecke, am_kleinen_wannsee, seglerweg, koblanckstr,
            liebermann_villa, am_grossen_wannsee, haus_der_wannsee_konferenz,
            zum_heckeshorn, strasse_zum_loewen, seglerweg, conradstr, am_kleinen_wannsee,
            wannseebruecke, wannsee,
        ],
    }
}
