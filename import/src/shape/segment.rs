use std::fmt;

use crate::coord::{Point, PointDebug};

#[derive(PartialEq, Clone)]
pub(crate) struct Segment {
    points: Vec<Point>,
}

impl Segment {
    pub(super) fn new(points: Vec<Point>) -> Self {
        Self { points }
    }

    pub(super) fn size(&self) -> usize {
        self.points.len()
    }

    pub(super) fn split(&mut self, at: usize) -> Segment {
        let split = self.points.split_off(at);
        Self::new(split)
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = Point> + DoubleEndedIterator + '_ {
        self.points.iter().copied()
    }
}

#[cfg(not(tarpaulin_include))]
impl fmt::Debug for Segment {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_list()
            .entries(
                self.points
                    .iter()
                    .map(|&position| PointDebug::new(position, 6)),
            )
            .finish()
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;
    use crate::coord::project;

    macro_rules! segments {
        ($( $segment:ident : [ $( $lat:expr, $lon:expr );* $(;)? ] ),* $(,)?) => {
            $(
                pub(crate) fn $segment() -> Segment {
                    Segment {
                        points: vec![ $( project($lat, $lon) ),* ],
                    }
                }
            )*
        }
    }

    segments! {
        westkreuz_stadtbahn: [
            52.502, 13.287; 52.502, 13.286; 52.501, 13.286; 52.501, 13.285; 52.501, 13.284;
            52.501, 13.283; 52.500, 13.282; 52.500, 13.281; 52.500, 13.280; 52.499, 13.279;
        ],
        westkreuz_ringbahn: [
            52.499, 13.287; 52.499, 13.286; 52.500, 13.285; 52.500, 13.284; 52.501, 13.284;
            52.502, 13.283; 52.503, 13.283; 52.503, 13.282; 52.504, 13.282; 52.505, 13.282;
        ],
        nollendorfplatz_innsbrucker_platz: [
            52.500, 13.354; 52.496, 13.343; 52.489, 13.340; 52.483, 13.342; 52.478, 13.343;
        ],
        clara_jaschke_str_landsberger_allee_petersburger_str: [
            52.525, 13.366; 52.526, 13.367; 52.526, 13.370; 52.529, 13.377; 52.530, 13.382; 52.532, 13.388;
            52.536, 13.390; 52.538, 13.396; 52.540, 13.401; 52.541, 13.406; 52.541, 13.412; 52.540, 13.420;
            52.539, 13.424; 52.538, 13.428; 52.536, 13.434; 52.534, 13.437; 52.532, 13.441; 52.528, 13.445;
            52.527, 13.447;
        ],
        clara_jaschke_str_hauptbahnhof: [
            52.525, 13.366; 52.526, 13.367;
        ],
        hauptbahnhof_lueneburger_str: [
            52.524, 13.363; 52.523, 13.362;
        ],
        hauptbahnhof_landsberger_allee_petersburger_str: [
            52.526, 13.370; 52.529, 13.377; 52.530, 13.382; 52.532, 13.388; 52.536, 13.390; 52.538, 13.396;
            52.540, 13.401; 52.541, 13.406; 52.541, 13.412; 52.540, 13.420; 52.539, 13.424; 52.538, 13.428;
            52.536, 13.434; 52.534, 13.437; 52.532, 13.441; 52.528, 13.445; 52.527, 13.447;
        ],
        landsberger_allee_petersburger_str_warschauer_str: [
            52.522, 13.450; 52.519, 13.453; 52.516, 13.454; 52.512, 13.452; 52.508, 13.450; 52.505, 13.448;
        ],
        landsberger_allee_petersburger_str_revaler_str: [
            52.522, 13.450; 52.519, 13.453; 52.516, 13.454; 52.512, 13.452;
        ],
        revaler_str: [
            52.509, 13.451;
        ],
        warschauer_str: [
            52.508, 13.450; 52.505, 13.448;
        ],
    }
}
