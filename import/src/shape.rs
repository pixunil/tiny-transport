use std::error::Error;
use std::collections::HashMap;
use std::iter::once;

use itertools::{Itertools, EitherOrBoth::*};

use na::Point2;

use super::utils::*;

pub type Shape = Vec<Point2<f32>>;

pub fn transform(lat: f32, lon: f32) -> Point2<f32> {
    Point2::new(lon, 2.0 * lat)
}

pub struct Importer;

impl Importer {
    pub fn import(dataset: &mut impl Dataset) -> Result<HashMap<Id, Shape>, Box<dyn Error>> {
        let mut shapes = HashMap::new();

        for result in dataset.read_csv("shapes.txt")?.deserialize() {
            let record: ShapeRecord = result?;
            let waypoint = transform(record.shape_pt_lat, record.shape_pt_lon);
            shapes.entry(record.shape_id)
                .or_insert_with(Shape::new)
                .push(waypoint);
        }

        let shapes = shapes.into_iter()
            .map(|(id, shape)| {
                (id, Self::smooth(shape))
            })
            .collect();

        Ok(shapes)
    }

    fn smooth(mut shape: Shape) -> Shape {
        shape.dedup();
        loop {
            let len = shape.len();
            shape = Self::remove_spikes(shape);
            shape = Self::smooth_zigzags(shape);
            if shape.len() == len {
                return shape;
            }
        }
    }

    fn remove_spikes(shape: Shape) -> Shape {
        let spike_angle = (120_f32).to_radians();
        let segments = shape.windows(2)
            .map(|adjacent| adjacent[1] - adjacent[0]);

        let cleaned = shape.iter()
            .skip(1)
            .zip_longest(segments.tuple_windows())
            .filter_map(|element| {
                match element {
                    Both(&waypoint, (before, after)) => {
                        if before.angle(&after) > spike_angle {
                            None
                        } else {
                            Some(waypoint)
                        }
                    },
                    Left(&waypoint) => Some(waypoint),
                    Right(_) => unreachable!(),
                }
            })
            .dedup();

        once(shape[0]).chain(cleaned).collect()
    }

    fn smooth_zigzags(shape: Shape) -> Shape {
        let zigzag_angle = (20_f32).to_radians();
        let segments = shape.windows(2)
            .map(|adjacent| adjacent[1] - adjacent[0]);

        let mut was_merged = false;
        let smoothed = shape.iter()
            .skip(1)
            .zip_longest(segments.tuple_windows())
            .filter_map(|element| {
                match element {
                    Both(&waypoint, (before, between, after)) => {
                        let angles = [
                            before.angle(&between),
                            between.angle(&after),
                            before.angle(&after),
                        ];

                        if was_merged {
                            was_merged = false;
                            None
                        } else if (angles[0] + angles[1] - angles[2]).abs() > zigzag_angle {
                            was_merged = true;
                            Some(waypoint + between * 0.5)
                        } else {
                            Some(waypoint)
                        }
                    },
                    Left(&waypoint) => Some(waypoint),
                    Right(_) => unreachable!(),
                }
            });

        once(shape[0]).chain(smoothed).collect()
    }
}

#[derive(Debug, Deserialize)]
struct ShapeRecord {
    shape_id: Id,
    shape_pt_lat: f32,
    shape_pt_lon: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[macro_export]
    macro_rules! shape {
        ($($lat:expr, $lon:expr);*) => (
            vec![$($crate::shape::transform($lat, $lon)),*]
        );
        (blue) => (
            $crate::shape!(52.526, 13.369; 52.523, 13.378; 52.520, 13.387; 52.521, 13.394; 52.523, 13.402)
        );
    }

    #[test]
    fn test_remove_overlapping() {
        let mut shape = shape!(blue);
        shape.insert(3, shape[2]);
        assert_eq!(Importer::smooth(shape), shape!(blue));
    }

    #[test]
    fn test_remove_spike() {
        let mut shape = shape!(blue);
        shape.insert(3, transform(13.392, 52.508));
        assert_eq!(Importer::smooth(shape), shape!(blue));
    }

    #[test]
    fn test_remove_jump() {
        let mut shape = shape!(blue);
        shape.insert(3, transform(13.386, 52.521));
        shape.insert(4, shape[2]);
        assert_eq!(Importer::smooth(shape), shape!(blue));
    }

    #[test]
    fn test_smooth_zigzag() {
        let mut shape = shape!(blue);
        let original = shape.remove(2);
        let offset = transform(0.0001, 0.0).coords;
        shape.insert(2, original + offset);
        shape.insert(3, original - offset);
        assert_eq!(Importer::smooth(shape), shape!(blue));
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = crate::dataset!(
            shapes:
                shape_id, shape_pt_lat, shape_pt_lon;
                1,        52.51,        13.37;
                1,        52.52,        13.37;
                2,        52.526,       13.369;
                2,        52.523,       13.378;
                2,        52.520,       13.387;
                2,        52.521,       13.394;
                2,        52.523,       13.402
        );

        let shapes = Importer::import(&mut dataset).unwrap();
        assert_eq!(shapes.len(), 2);
        assert_eq!(shapes["1"], shape!(52.51, 13.37; 52.52, 13.37));
        assert_eq!(shapes["2"], shape!(blue));
    }
}
