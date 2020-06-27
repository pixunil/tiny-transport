use na::Point2;

use crate::line;

#[derive(Debug, PartialEq)]
pub struct Station {
    position: Point2<f32>,
    name: String,
    kind: Kind,
}

impl Station {
    pub fn new(position: Point2<f32>, name: String, kind: Kind) -> Station {
        Station {
            position,
            name,
            kind,
        }
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

    pub fn fill_type_buffer(&self, buffer: &mut Vec<u8>) {
        buffer.push(self.kind as u8);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    BusStop = 0,
    TramStop = 1,
    FerryPier = 2,
    Interchange = 3,
}

impl Kind {
    pub fn from_line_kinds(kinds: &[line::Kind]) -> Self {
        if kinds.contains(&line::Kind::Railway)
            || kinds.contains(&line::Kind::SuburbanRailway)
            || kinds.contains(&line::Kind::UrbanRailway)
        {
            Self::Interchange
        } else if kinds.contains(&line::Kind::Tram) {
            Self::TramStop
        } else if kinds.contains(&line::Kind::Bus) {
            Self::BusStop
        } else if kinds.contains(&line::Kind::WaterTransport) {
            Self::FerryPier
        } else {
            unreachable!()
        }
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use super::*;

    macro_rules! stations {
        ($($station:ident: $x:expr, $y:expr, $kind:ident, $name:expr);* $(;)?) => {
            $(
                pub fn $station() -> Station {
                    Station {
                        position: Point2::new($x as f32, $y as f32),
                        name: $name.to_string(),
                        kind: Kind::$kind,
                    }
                }
            )*
        }
    }

    stations! {
        hauptbahnhof:                         -1385,  -1812, Interchange, "Hauptbahnhof";
        friedrichstr:                          -168,  -1147, Interchange, "Friedrichstr.";
        hackescher_markt:                       846,  -1428, Interchange, "Hackescher Markt";
        bellevue:                             -2893,  -1178, Interchange, "Bellevue";
        naturkundemuseum:                      -491,  -2348, Interchange, "Naturkundemuseum";
        franzoesische_str:                      -55,   -558, Interchange, "Französische Str.";
        oranienburger_tor:                     -124,  -1632, Interchange, "Oranienburger Tor";
        universitaetsstr:                       147,   -995, TramStop,    "Universitätsstr.";
        am_kupfergraben:                        389,  -1039, TramStop,    "Am Kupfergraben";
        georgenstr_am_kupfergraben:             308,  -1160, TramStop,    "Georgenstr./Am Kupfergraben";
        zingster_str:                          7269,  -6742, TramStop,    "Zingster Str.";
        zingster_str_ribnitzer_str:            7400,  -6517, TramStop,    "Zingster Str./Ribnitzer Str.";
        ahrenshooper_str:                      7730,  -6065, TramStop,    "Ahrenshooper Str.";
        prerower_platz:                        7926,  -5727, TramStop,    "Prerower Platz";
        weskammstr:                           -2958,  10616, BusStop,     "Weskammstr.";
        lichterfelder_ring_waldsassener_str:  -2963,  10838, BusStop,     "Lichterfelder Ring/Waldsassener Str.";
        waldsassener_str:                     -2906,  11285, BusStop,     "Waldsassener Str.";
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::fixtures::stations;

    #[test]
    fn test_station_kind_from_line_kinds() {
        assert_eq!(
            Kind::from_line_kinds(&[line::Kind::SuburbanRailway]),
            Kind::Interchange
        );
        assert_eq!(Kind::from_line_kinds(&[line::Kind::Tram]), Kind::TramStop);
        assert_eq!(
            Kind::from_line_kinds(&[line::Kind::Tram, line::Kind::Bus]),
            Kind::TramStop
        );
        assert_eq!(Kind::from_line_kinds(&[line::Kind::Bus]), Kind::BusStop);
    }

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

    #[test]
    fn test_station_types() {
        let mut buffer = Vec::new();
        stations::hauptbahnhof().fill_type_buffer(&mut buffer);
        stations::am_kupfergraben().fill_type_buffer(&mut buffer);
        stations::weskammstr().fill_type_buffer(&mut buffer);
        assert_eq!(*buffer, [3, 1, 0])
    }
}
