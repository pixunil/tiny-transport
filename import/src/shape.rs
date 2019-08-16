use std::error::Error;
use std::rc::Rc;
use std::collections::HashMap;

use na::Point2;

use super::utils::*;

pub type Shape = Vec<Point2<f32>>;

pub struct Importer;

impl Importer {
    pub fn import(dataset: &mut impl Dataset) -> Result<HashMap<Id, Rc<Shape>>, Box<dyn Error>> {
        let mut shapes = HashMap::new();

        for result in dataset.read_csv("shapes.txt")?.deserialize() {
            let record: ShapeRecord = result?;
            let waypoint = Point2::new(record.shape_pt_lon, record.shape_pt_lat);
            shapes.entry(record.shape_id)
                .or_insert_with(Shape::new)
                .push(waypoint);
        }

        let shapes = shapes.into_iter()
            .map(|(id, shape)| {
                let shape = Self::smooth(shape);
                (id, Rc::new(shape))
            })
            .collect();

        Ok(shapes)
    }

    fn smooth(mut shape: Shape) -> Shape {
        loop {
            let original_length = shape.len();
            shape = Self::smooth_pass(shape);
            if shape.len() == original_length {
                return shape;
            }
        }
    }

    fn smooth_pass(shape: Shape) -> Shape {
        let segments = shape.windows(2)
            .map(|segment| segment[1] - segment[0])
            .collect::<Vec<_>>();
        let mut waypoints = shape.into_iter();
        let start = waypoints.next().unwrap();
        let end = waypoints.next_back().unwrap();
        let mut shape = waypoints
            .zip(segments.windows(2))
            .filter(|(_, adjacent)| {
                adjacent[0].norm() == 0.0 || adjacent[0].perp(&adjacent[1]) != 0.0
            })
            .filter(|(_, adjacent)| {
                let angle = adjacent[0].angle(&adjacent[1]);
                angle < (120f32).to_radians()
            })
            .map(|(waypoint, _)| waypoint)
            .collect::<Vec<_>>();
        shape.insert(0, start);
        shape.push(end);
        shape
    }
}

#[derive(Debug, Deserialize)]
struct ShapeRecord {
    shape_id: Id,
    shape_pt_lat: f32,
    shape_pt_lon: f32,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[macro_export]
    macro_rules! shape {
        ($($lat:expr, $lon:expr);*) => (
            vec![$(::na::Point2::new($lat, $lon)),*]
        );
        (blue) => (
            $crate::shape!(13.37, 52.52; 13.37, 52.53; 13.38, 52.53)
        )
    }

    #[test]
    fn test_remove_overlapping() {
        let shape = shape!(13.37, 52.52; 13.37, 52.53; 13.37, 52.53; 13.38, 52.53);
        assert_eq!(Importer::smooth(shape), shape!(blue));
    }

    #[test]
    fn test_remove_on_segment() {
        let shape = shape!(13.37, 52.52; 13.37, 52.525; 13.37, 52.53; 13.38, 52.53);
        assert_eq!(Importer::smooth(shape), shape!(blue));
    }

    #[test]
    fn test_remove_spike() {
        let shape = shape!(13.37, 52.52; 13.0, 52.525; 13.37, 52.53; 13.38, 52.53);
        assert_eq!(Importer::smooth(shape), shape!(blue));
    }

    #[test]
    fn test_remove_jump() {
        let shape = shape!(13.37, 52.52; 13.37, 52.51; 13.37, 52.52; 13.37, 52.53; 13.38, 52.53);
        assert_eq!(Importer::smooth(shape), shape!(blue));
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = crate::dataset!(
            shapes:
                shape_id, shape_pt_lat, shape_pt_lon;
                1,        52.51,        13.37;
                1,        52.52,        13.37;
                2,        52.52,        13.37;
                2,        52.53,        13.37;
                2,        52.53,        13.38
        );

        let shapes = Importer::import(&mut dataset).unwrap();
        assert_eq!(shapes.len(), 2);
        assert_eq!(*shapes["1"], shape!(13.37, 52.51; 13.37, 52.52));
        assert_eq!(*shapes["2"], shape!(blue));
    }
}
