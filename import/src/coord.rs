use std::fmt;

use geomorph::coord::Coord;
use geomorph::utm::Utm;
use na::Point2;

pub type Point = Point2<f64>;

pub fn project(lat: f64, lon: f64) -> Point {
    let coord = Coord::new(lat, lon);
    let utm = Utm::from(coord);
    Point::new(utm.easting, utm.northing)
}

pub fn project_back(position: Point) -> (f64, f64) {
    let utm = Utm::new(position.x, position.y, true, 33, 'U', false);
    let coord = Coord::from(utm);
    (coord.lat, coord.lon)
}

pub fn transform(point: Point) -> Point2<f32> {
    let translated = point.coords - project(52.51, 13.39).coords;
    Point2::new(translated.x.round() as f32, -translated.y.round() as f32)
}

pub(crate) struct PointDebug {
    position: Point,
    precision: usize,
}

#[cfg(not(tarpaulin_include))]
impl PointDebug {
    pub(crate) fn new(position: Point, precision: usize) -> Self {
        PointDebug {
            position,
            precision,
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl fmt::Debug for PointDebug {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let (lat, lon) = project_back(self.position);
        write!(
            formatter,
            "({:.precision$}, {:.precision$})",
            lat,
            lon,
            precision = self.precision
        )
    }
}
