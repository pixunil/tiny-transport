use serde_derive::{Deserialize, Serialize};

use na::Point2;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Station {
    position: Point2<f32>,
    name: String,
}

impl Station {
    pub fn new(position: Point2<f32>, name: String) -> Station {
        Station { position, name }
    }

    pub fn load(self, kind: simulation::station::Kind) -> simulation::Station {
        simulation::Station::new(self.position, self.name, kind)
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use super::*;

    macro_rules! stations {
        ($($station:ident: $x:expr, $y:expr, $name:expr);* $(;)?) => {
            $(
                pub fn $station() -> Station {
                    Station::new(Point2::new($x as f32, $y as f32), $name.to_string())
                }
            )*
        }
    }

    stations! {
        hauptbahnhof:                         -1385,  -1812, "Hauptbahnhof";
        friedrichstr:                          -168,  -1147, "Friedrichstr.";
        hackescher_markt:                       846,  -1428, "Hackescher Markt";
        bellevue:                             -2893,  -1178, "Bellevue";
        naturkundemuseum:                      -491,  -2348, "Naturkundemuseum";
        franzoesische_str:                      -55,   -558, "Französische Str.";
        oranienburger_tor:                     -124,  -1632, "Oranienburger Tor";
        universitaetsstr:                       147,   -995, "Universitätsstr.";
        am_kupfergraben:                        389,  -1039, "Am Kupfergraben";
        georgenstr_am_kupfergraben:             308,  -1160, "Georgenstr./Am Kupfergraben";
    }
}

#[cfg(test)]
mod tests {
    use crate::station::fixtures as stations;

    #[test]
    fn test_load() {
        let station = stations::hauptbahnhof();
        assert_eq!(
            station.load(simulation::station::Kind::Interchange),
            simulation::fixtures::stations::hauptbahnhof()
        );
    }
}
