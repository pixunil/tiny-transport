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
