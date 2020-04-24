use geomorph::coord::Coord;
use geomorph::utm::Utm;

use na::Point2;

pub(crate) type Point = Point2<f64>;

pub(crate) fn project(lat: f64, lon: f64) -> Point {
    let coord = Coord::new(lat, lon);
    let utm = Utm::from(coord);
    Point::new(utm.easting, utm.northing)
}

fn project_back(position: Point) -> (f64, f64) {
    let utm = Utm::new(position.x, position.y, true, 33, 'U', false);
    let coord = Coord::from(utm);
    (coord.lat, coord.lon)
}

pub(crate) fn transform(point: Point) -> Point2<f32> {
    let translated = point.coords - project(52.52, 13.5).coords;
    Point2::new(translated.x as f32, -translated.y as f32)
}

pub(crate) fn debug_position(position: Point, alternate: bool) -> String {
    let (lat, lon) = project_back(position);
    format!(
        "({:.precision$}, {:.precision$})",
        lat,
        lon,
        precision = if alternate { 6 } else { 3 }
    )
}
