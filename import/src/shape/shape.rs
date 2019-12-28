use na::Point2;

use crate::create_id_type;

create_id_type!(ShapeId);

pub(crate) type Shape = Vec<Point2<f32>>;

pub(crate) fn transform(lat: f32, lon: f32) -> Point2<f32> {
    Point2::new(lon, 2.0 * lat)
}

#[cfg(test)]
mod tests {
    #[macro_export]
    macro_rules! shape {
        ($($lat:expr, $lon:expr);*) => (
            vec![$($crate::shape::transform($lat, $lon)),*]
        );
        (blue) => (
            $crate::shape!(52.526, 13.369; 52.523, 13.378; 52.520, 13.387; 52.521, 13.394; 52.523, 13.402)
        );
    }
}
