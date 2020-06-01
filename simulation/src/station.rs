use na::Point2;

#[derive(Debug, PartialEq)]
pub struct Station {
    position: Point2<f32>,
    name: String,
}

impl Station {
    pub fn new(position: Point2<f32>, name: String) -> Station {
        Station { position, name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn contains(&self, position: Point2<f32>) -> bool {
        na::distance(&self.position, &position) <= 45.0
    }

    pub fn fill_vertice_buffer(&self, buffer: &mut Vec<f32>) {
        buffer.extend(self.position.iter())
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
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::fixtures::*;

    #[test]
    fn test_getters() {
        let station = stations::hauptbahnhof();
        assert_eq!(station.name(), "Hauptbahnhof");
    }

    #[test]
    fn test_station_contains_center() {
        let station = stations::hauptbahnhof();
        assert!(station.contains(Point2::new(-1385.0, -1812.0)));
    }

    #[test]
    fn test_station_contains_border() {
        let station = stations::hauptbahnhof();
        for &position in &[
            Point2::new(-1340.0, -1812.0),
            Point2::new(-1385.0, -1768.0),
            Point2::new(-1430.0, -1812.0),
            Point2::new(-1385.0, -1857.0),
        ] {
            assert!(station.contains(position));
        }
    }

    #[test]
    fn test_station_excludes_outside() {
        let station = stations::hauptbahnhof();
        for &position in &[
            Point2::new(-1340.0, -1768.0),
            Point2::new(-1335.0, -1812.0),
            Point2::new(-1385.0, -1763.0),
        ] {
            assert!(!station.contains(position));
        }
    }

    #[test]
    fn test_station_vertices() {
        let station = stations::hauptbahnhof();
        let mut buffer = Vec::new();
        station.fill_vertice_buffer(&mut buffer);
        assert_relative_eq!(*buffer, [-1385.0, -1812.0])
    }
}
