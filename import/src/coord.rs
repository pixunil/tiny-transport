use geomorph::coord::Coord;
use geomorph::utm::Utm;

use na::Point2;

pub(crate) type Point = Point2<f64>;

pub(crate) fn project(lat: f64, lon: f64) -> Point {
    let coord = Coord::new(lat, lon);
    let utm = Utm::from(coord);
    Point::new(utm.easting, utm.northing)
}

pub(crate) fn transform(point: Point) -> Point2<f32> {
    let mut translated = point.coords - project(52.52, 13.5).coords;
    translated *= 0.03;
    Point2::new(translated.x as f32, -translated.y as f32)
}
