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

#[cfg(not(tarpaulin_include))]
pub(crate) fn debug_position(position: Point, alternate: bool) -> String {
    let (lat, lon) = project_back(position);
    format!(
        "({:.precision$}, {:.precision$})",
        lat,
        lon,
        precision = if alternate { 6 } else { 3 }
    )
}
