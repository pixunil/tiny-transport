pub(crate) use crate::agency::fixtures::agencies;
pub(crate) use crate::line::fixtures::{incomplete_lines, lines};
pub(crate) use crate::location::fixtures::locations;
pub(crate) use crate::service::fixtures::services;
pub(crate) use crate::shape::fixtures::shapes;
pub(crate) use crate::trip::fixtures::{
    nodes, route_buffers, route_variants, routes, trip_buffers, trips,
};

macro_rules! stop_locations {
    ($($line:ident: {$($route:ident => [$($location:ident),* $(,)?]),* $(,)?}),* $(,)?) => (
        pub(crate) mod stop_locations {
            $(
                pub(crate) mod $line {
                    use std::rc::Rc;
                    use crate::fixtures::locations;
                    use crate::location::Location;

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
    s41: {
        circle => [
            gesundbrunnen, ostkreuz, suedkreuz, westkreuz, gesundbrunnen,
        ],
    },
    u4: {
        empty => [],
        nollendorfplatz_innsbrucker_platz => [
            nollendorfplatz, viktoria_luise_platz, bayerischer_platz,
            rathaus_schoeneberg, innsbrucker_platz,
        ],
    },
    tram_m10: {
        clara_jaschke_str_warschauer_str => [
            clara_jaschke_str, hauptbahnhof, invalidenpark, naturkundemuseum, nordbahnhof,
            gedenkstaette_berliner_mauer, bernauer_str, wolliner_str,
            friedrich_ludwig_jahn_sportpark, eberswalder_str, husemannstr,
            prenzlauer_allee_danziger_str, winsstr, greifswalder_str_danziger_str,
            arnswalder_platz, kniprodestr_danziger_str, paul_heyse_str,
            landsberger_allee_petersburger_str, strassmannstr, bersarinplatz, frankfurter_tor,
            gruenberger_str_warschauer_str, warschauer_str, warschauer_str,
        ],
        warschauer_str_lueneburger_str => [
            warschauer_str, warschauer_str, revaler_str, gruenberger_str_warschauer_str,
            frankfurter_tor, bersarinplatz, strassmannstr, landsberger_allee_petersburger_str,
            paul_heyse_str, kniprodestr_danziger_str, arnswalder_platz,
            greifswalder_str_danziger_str, winsstr, prenzlauer_allee_danziger_str, husemannstr,
            eberswalder_str, friedrich_ludwig_jahn_sportpark, wolliner_str, bernauer_str,
            gedenkstaette_berliner_mauer, nordbahnhof, naturkundemuseum, invalidenpark,
            hauptbahnhof, lesser_ury_weg, lueneburger_str,
        ],
        clara_jaschke_str_landsberger_allee_petersburger_str => [
            clara_jaschke_str, hauptbahnhof, invalidenpark, naturkundemuseum, nordbahnhof,
            gedenkstaette_berliner_mauer, bernauer_str, wolliner_str,
            friedrich_ludwig_jahn_sportpark, eberswalder_str, husemannstr,
            prenzlauer_allee_danziger_str, winsstr, greifswalder_str_danziger_str,
            arnswalder_platz, kniprodestr_danziger_str, paul_heyse_str,
            landsberger_allee_petersburger_str,
        ],
        landsberger_allee_petersburger_str_lueneburger_str => [
            landsberger_allee_petersburger_str, paul_heyse_str, kniprodestr_danziger_str,
            arnswalder_platz, greifswalder_str_danziger_str, winsstr, prenzlauer_allee_danziger_str,
            husemannstr, eberswalder_str, friedrich_ludwig_jahn_sportpark, wolliner_str,
            bernauer_str, gedenkstaette_berliner_mauer, nordbahnhof, naturkundemuseum,
            invalidenpark, hauptbahnhof, lesser_ury_weg, lueneburger_str,
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
            wannsee, wannsee, wannseebruecke, am_kleinen_wannsee, seglerweg, koblanckstr,
            liebermann_villa, am_grossen_wannsee, haus_der_wannsee_konferenz,
            zum_heckeshorn, strasse_zum_loewen, seglerweg, conradstr, am_kleinen_wannsee,
            wannseebruecke, wannsee, wannsee,
        ],
    }
}
