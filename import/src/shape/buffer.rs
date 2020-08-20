use std::fmt;

use crate::coord::{Point, PointDebug};
use crate::create_id_type;

create_id_type!(ShapeId);

#[derive(PartialEq)]
pub(super) struct Buffer {
    points: Vec<Point>,
}

impl Buffer {
    pub(super) fn new() -> Self {
        Self { points: Vec::new() }
    }

    pub(super) fn add(&mut self, position: Point) {
        self.points.push(position);
    }

    #[cfg(test)]
    pub(super) fn reversed(mut self) -> Self {
        self.points.reverse();
        self
    }
}

impl From<Vec<Point>> for Buffer {
    fn from(value: Vec<Point>) -> Self {
        Self { points: value }
    }
}

impl IntoIterator for Buffer {
    type Item = Point;
    type IntoIter = <Vec<Point> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.points.into_iter()
    }
}

#[cfg(not(tarpaulin_include))]
impl fmt::Debug for Buffer {
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
    macro_rules! shapes {
        ($(
            $line:ident: {
                $(
                    $shape:ident : [$($lat:expr, $lon:expr);* $(;)?]
                ),* $(,)?
            }
        ),* $(,)?) => {
            use crate::shape::Buffer;

            $(
                pub(in crate::shape) mod $line {
                    use super::*;
                    use crate::coord::project;

                    $(
                        pub(in crate::shape) fn $shape() -> Buffer {
                            Buffer {
                                points: vec![$( project($lat, $lon) ),*],
                            }
                        }
                    )*
                }
            )*
        };
    }

    shapes! {
        s3: {
            westkreuz_outbound: [
                52.502, 13.287; 52.502, 13.286; 52.501, 13.286; 52.501, 13.285; 52.501, 13.284;
                52.501, 13.283; 52.500, 13.282; 52.500, 13.281; 52.500, 13.280; 52.499, 13.279;
            ],
            westkreuz_inbound: [
                52.499, 13.279; 52.500, 13.280; 52.500, 13.281; 52.500, 13.282; 52.501, 13.283;
                52.501, 13.284; 52.501, 13.285; 52.501, 13.286; 52.502, 13.286; 52.502, 13.287;
            ],
        },
        s41: {
            westkreuz_anticlockwise: [
                52.499, 13.287; 52.499, 13.286; 52.500, 13.285; 52.500, 13.284; 52.501, 13.284;
                52.502, 13.283; 52.503, 13.283; 52.503, 13.282; 52.504, 13.282; 52.505, 13.282;
            ],
        },
        s42: {
            westkreuz_clockwise: [
                52.505, 13.282; 52.504, 13.282; 52.503, 13.282; 52.503, 13.283; 52.502, 13.283;
                52.501, 13.284; 52.500, 13.284; 52.500, 13.285; 52.499, 13.286; 52.499, 13.287;
            ],
        },
        u4: {
            nollendorfplatz_innsbrucker_platz: [
                52.500, 13.354; 52.496, 13.343; 52.489, 13.340; 52.483, 13.342; 52.478, 13.343;
            ],
            innsbrucker_platz_nollendorfplatz: [
                52.478, 13.343; 52.483, 13.342; 52.489, 13.340; 52.496, 13.343; 52.500, 13.354;
            ],
        },
        tram_m10: {
            clara_jaschke_str_warschauer_str: [
                52.525, 13.366; 52.526, 13.367; 52.526, 13.370; 52.529, 13.377; 52.530, 13.382;
                52.532, 13.388; 52.536, 13.390; 52.538, 13.396; 52.540, 13.401; 52.541, 13.406;
                52.541, 13.412; 52.540, 13.420; 52.539, 13.424; 52.538, 13.428; 52.536, 13.434;
                52.534, 13.437; 52.532, 13.441; 52.528, 13.445; 52.527, 13.447; 52.522, 13.450;
                52.519, 13.453; 52.516, 13.454; 52.512, 13.452; 52.508, 13.450; 52.505, 13.448;
            ],
            warschauer_str_lueneburger_str: [
                52.505, 13.448; 52.508, 13.450; 52.509, 13.451; 52.512, 13.452; 52.516, 13.454;
                52.519, 13.453; 52.522, 13.450; 52.527, 13.447; 52.528, 13.445; 52.532, 13.441;
                52.534, 13.437; 52.536, 13.434; 52.538, 13.428; 52.539, 13.424; 52.540, 13.420;
                52.541, 13.412; 52.541, 13.406; 52.540, 13.401; 52.538, 13.396; 52.536, 13.390;
                52.532, 13.388; 52.530, 13.382; 52.529, 13.377; 52.526, 13.370; 52.524, 13.363;
                52.523, 13.362;
            ],
            clara_jaschke_str_landsberger_allee_petersburger_str: [
                52.525, 13.366; 52.526, 13.367; 52.526, 13.370; 52.529, 13.377; 52.530, 13.382;
                52.532, 13.388; 52.536, 13.390; 52.538, 13.396; 52.540, 13.401; 52.541, 13.406;
                52.541, 13.412; 52.540, 13.420; 52.539, 13.424; 52.538, 13.428; 52.536, 13.434;
                52.534, 13.437; 52.532, 13.441; 52.528, 13.445; 52.527, 13.447;
            ],
            landsberger_allee_petersburger_str_lueneburger_str: [
                52.527, 13.447; 52.528, 13.445; 52.532, 13.441; 52.534, 13.437; 52.536, 13.434;
                52.538, 13.428; 52.539, 13.424; 52.540, 13.420; 52.541, 13.412; 52.541, 13.406;
                52.540, 13.401; 52.538, 13.396; 52.536, 13.390; 52.532, 13.388; 52.530, 13.382;
                52.529, 13.377; 52.526, 13.370; 52.524, 13.363; 52.523, 13.362;
            ],
        },
    }
}
