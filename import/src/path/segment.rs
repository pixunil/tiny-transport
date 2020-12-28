use crate::path::Node;

#[derive(Debug, PartialEq)]
pub struct Segment {
    nodes: Vec<Node>,
}

impl Segment {
    pub(super) fn new(nodes: Vec<Node>) -> Self {
        Self { nodes }
    }

    pub(crate) fn nodes(&self) -> &[Node] {
        &self.nodes
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use std::rc::Rc;

    use super::*;
    use crate::coord::project;
    use crate::fixtures::locations;

    macro_rules! segments {
        (@kind $location:ident) => ( Some(Rc::new(locations::$location())) );
        (@kind) => ( None );
        ($( $segment:ident : [ $( $lat:expr, $lon:expr $( , $location:ident )? );* $(;)? ] ),* $(,)?) => {
            $(
                pub(crate) fn $segment() -> Segment {
                    Segment {
                        nodes: vec![ $(
                            Node::new(project($lat, $lon), segments!(@kind $($location)?))
                        ),* ],
                    }
                }
            )*
        }
    }

    segments! {
        circle: [
            52.549, 13.388, gesundbrunnen;
            52.503, 13.469, ostkreuz;
            52.475, 13.366, suedkreuz;
            52.501, 13.283, westkreuz;
            52.549, 13.388, gesundbrunnen;
        ],
        hackescher_markt_bellevue: [
            52.523, 13.402, hackescher_markt;
            52.521, 13.386, friedrichstr;
            52.525, 13.369, hauptbahnhof;
            52.520, 13.347, bellevue;
        ],
        clara_jaschke_str_landsberger_allee_petersburger_str: [
            52.525, 13.366, clara_jaschke_str;
            52.526, 13.367;
            52.526, 13.370, hauptbahnhof;
            52.529, 13.377, invalidenpark;
            52.530, 13.382, naturkundemuseum;
            52.532, 13.388, nordbahnhof;
            52.536, 13.390, gedenkstaette_berliner_mauer;
            52.538, 13.396, bernauer_str;
            52.540, 13.401, wolliner_str;
            52.541, 13.406, friedrich_ludwig_jahn_sportpark;
            52.541, 13.412, eberswalder_str;
            52.540, 13.420, husemannstr;
            52.539, 13.424, prenzlauer_allee_danziger_str;
            52.538, 13.428, winsstr;
            52.536, 13.434, greifswalder_str_danziger_str;
            52.534, 13.437, arnswalder_platz;
            52.532, 13.441, kniprodestr_danziger_str;
            52.528, 13.445, paul_heyse_str;
            52.527, 13.447, landsberger_allee_petersburger_str;
        ],
        clara_jaschke_str_hauptbahnhof: [
            52.525, 13.366, clara_jaschke_str;
            52.526, 13.367;
        ],
        hauptbahnhof_lueneburger_str: [
            52.524, 13.363, lesser_ury_weg;
            52.523, 13.362, lueneburger_str;
        ],
        hauptbahnhof_landsberger_allee_petersburger_str: [
            52.526, 13.370, hauptbahnhof;
            52.529, 13.377, invalidenpark;
            52.530, 13.382, naturkundemuseum;
            52.532, 13.388, nordbahnhof;
            52.536, 13.390, gedenkstaette_berliner_mauer;
            52.538, 13.396, bernauer_str;
            52.540, 13.401, wolliner_str;
            52.541, 13.406, friedrich_ludwig_jahn_sportpark;
            52.541, 13.412, eberswalder_str;
            52.540, 13.420, husemannstr;
            52.539, 13.424, prenzlauer_allee_danziger_str;
            52.538, 13.428, winsstr;
            52.536, 13.434, greifswalder_str_danziger_str;
            52.534, 13.437, arnswalder_platz;
            52.532, 13.441, kniprodestr_danziger_str;
            52.528, 13.445, paul_heyse_str;
            52.527, 13.447, landsberger_allee_petersburger_str;
        ],
        landsberger_allee_petersburger_str_warschauer_str: [
            52.522, 13.450, strassmannstr;
            52.519, 13.453, bersarinplatz;
            52.516, 13.454, frankfurter_tor;
            52.512, 13.452, gruenberger_str_warschauer_str;
            52.509, 13.451, revaler_str;
            52.508, 13.450, warschauer_str;
            52.505, 13.448, warschauer_str;
        ],
        landsberger_allee_petersburger_str_revaler_str: [
            52.527, 13.447, landsberger_allee_petersburger_str;
            52.522, 13.450, strassmannstr;
            52.519, 13.453, bersarinplatz;
            52.516, 13.454, frankfurter_tor;
            52.512, 13.452, gruenberger_str_warschauer_str
        ],
        revaler_str: [
            52.509, 13.451, revaler_str;
        ],
        warschauer_str: [
            52.508, 13.450, warschauer_str;
            52.505, 13.448, warschauer_str;
        ],
        strassmannstr_warschauer_str_too_few_points: [
            52.522, 13.450, strassmannstr;
            52.519, 13.453, bersarinplatz;
            52.516, 13.454, frankfurter_tor;
            52.512, 13.452, gruenberger_str_warschauer_str;
            52.508, 13.450, warschauer_str;
            52.508, 13.450, warschauer_str;
        ],
        oranienburger_tor_friedrichstr: [
            52.525, 13.388, oranienburger_tor;
            52.524, 13.388;
            52.521, 13.388;
            52.520, 13.388, friedrichstr;
            52.519, 13.389;
            52.519, 13.390;
        ],
        universitaetsstr_am_kupfergraben: [
            52.519, 13.391;
            52.519, 13.392, universitaetsstr;
            52.519, 13.396, am_kupfergraben;
        ],
        am_kupfergraben_georgenstr: [
            52.519, 13.396, am_kupfergraben;
            52.520, 13.396;
            52.521, 13.395;
            52.521, 13.394;
            52.520, 13.393, georgenstr_am_kupfergraben;
            52.520, 13.391;
            52.520, 13.390;
        ],
        anhalter_bahnhof_tiergarten: [
            52.505, 13.382, anhalter_bahnhof;
            52.506, 13.380;
            52.507, 13.380, abgeordnetenhaus;
            52.507, 13.379;
            52.508, 13.378;
            52.509, 13.377, potsdamer_platz_bus_stresemannstr;
            52.510, 13.377, potsdamer_platz_vossstr;
            52.511, 13.377;
            52.512, 13.377;
            52.512, 13.376;
            52.512, 13.374;
            52.511, 13.372;
            52.511, 13.371;
            52.512, 13.371;
            52.513, 13.371;
            52.514, 13.371;
            52.516, 13.371;
        ],
        tiergarten_hauptbahnhof: [
            52.518, 13.372;
            52.519, 13.372;
            52.520, 13.373;
            52.521, 13.373;
            52.521, 13.372;
            52.5257,13.368, hauptbahnhof;
            52.526, 13.368, hauptbahnhof;
            52.527, 13.368;
            52.528, 13.368;
            52.527, 13.369;
        ],
        hauptbahnhof_tiergarten: [
            52.527, 13.369;
            52.526, 13.369, hauptbahnhof;
            52.5262,13.368, hauptbahnhof;
            52.522, 13.372;
            52.521, 13.372;
            52.520, 13.372;
            52.518, 13.371;
        ],
        wannsee_heckeshorn_wannsee: [
            52.422, 13.178, wannsee;
            52.421, 13.178, wannsee;
            52.421, 13.177;
            52.421, 13.176;
            52.420, 13.175, wannseebruecke;
            52.420, 13.174;
            52.421, 13.174;
            52.421, 13.173;
            52.421, 13.172;
            52.421, 13.171;
            52.421, 13.170;
            52.421, 13.169;
            52.421, 13.168;
            52.421, 13.167, am_kleinen_wannsee;
            52.421, 13.166;
            52.421, 13.165;
            52.422, 13.165;
            52.422, 13.164;
            52.423, 13.163;
            52.423, 13.162;
            52.424, 13.162, seglerweg;
            52.425, 13.161;
            52.426, 13.161;
            52.427, 13.162, koblanckstr;
            52.428, 13.162;
            52.428, 13.163;
            52.429, 13.164, liebermann_villa;
            52.430, 13.164;
            52.430, 13.165;
            52.431, 13.165;
            52.432, 13.165, am_grossen_wannsee;
            52.433, 13.164, haus_der_wannsee_konferenz;
            52.432, 13.163;
            52.432, 13.162;
            52.431, 13.162;
            52.431, 13.161;
            52.430, 13.161, zum_heckeshorn;
            52.429, 13.160;
            52.428, 13.160;
            52.427, 13.160, strasse_zum_loewen;
            52.427, 13.159;
            52.426, 13.160;
            52.424, 13.160, seglerweg;
            52.421, 13.162;
            52.420, 13.162, conradstr;
            52.420, 13.166;
            52.420, 13.167, am_kleinen_wannsee;
            52.421, 13.168;
            52.421, 13.170;
            52.421, 13.171;
            52.421, 13.172;
            52.421, 13.173;
            52.420, 13.174;
            52.420, 13.175, wannseebruecke;
            52.420, 13.176;
            52.421, 13.176;
            52.421, 13.177;
            52.421, 13.178, wannsee;
            52.422, 13.179, wannsee;
            52.422, 13.180;
            52.422, 13.179;
            52.422, 13.178;
        ],
    }
}
