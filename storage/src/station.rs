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

    pub fn load(self) -> simulation::Station {
        simulation::Station::new(self.position, self.name)
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
        oranienburger_tor:                     -124,  -1632, "Oranienburger Tor";
        universitaetsstr:                       147,   -995, "Universitätsstr.";
        am_kupfergraben:                        389,  -1039, "Am Kupfergraben";
        georgenstr_am_kupfergraben:             308,  -1160, "Georgenstr./Am Kupfergraben";
    }

    #[macro_export]
    macro_rules! stations_with_ids {
        ($( $station:ident ),* $(,)?) => ({
            let stations = vec![
                $( std::rc::Rc::new(simulation::fixtures::stations::$station() )),*
            ];
            let station_ids = vec![ $( stringify!($station) ),* ]
                .into_iter()
                .enumerate()
                .map(|(i, identifier)| (identifier, i))
                .collect::<std::collections::HashMap<_, _>>();
            (stations, station_ids)
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::station::fixtures as stations;

    #[test]
    fn test_load() {
        let station = stations::hauptbahnhof();
        assert_eq!(
            station.load(),
            simulation::fixtures::stations::hauptbahnhof()
        );
    }
}
