use std::collections::HashMap;
use std::error::Error;
use std::iter::once;
use std::time::Instant;

use itertools::{EitherOrBoth::*, Itertools};

use console::Term;

use indicatif::ProgressIterator;

use super::{Shape, ShapeId, ShapeRecord};
use crate::utils::progress::{elapsed, percent_bar};
use crate::utils::Dataset;

pub(crate) struct Importer;

impl Importer {
    pub(crate) fn import(
        dataset: &mut impl Dataset,
    ) -> Result<HashMap<ShapeId, Shape>, Box<dyn Error>> {
        let mut shapes = HashMap::new();

        let records = dataset.read_csv("shapes.txt", "Importing shapes")?;
        let started = Instant::now();
        for result in records {
            let record: ShapeRecord = result?;
            record.import(&mut shapes);
        }

        eprintln!(
            "Imported {} shapes in {:.2}s",
            shapes.len(),
            elapsed(started)
        );

        let started = Instant::now();
        let progress_bar = percent_bar(shapes.len() as u64, "Smoothing shapes");
        let shapes = shapes
            .into_iter()
            .progress_with(progress_bar)
            .map(|(id, shape)| (id, Self::smooth(shape)))
            .collect();

        Term::stderr().clear_last_lines(1).unwrap();
        eprintln!(
            "Smoothed shapes in {:.2}s",
            started.elapsed().as_millis() as f64 / 1000.0
        );
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
        let spike_angle = 120_f64.to_radians();
        let segments = shape.windows(2).map(|adjacent| adjacent[1] - adjacent[0]);

        let cleaned = shape
            .iter()
            .skip(1)
            .zip_longest(segments.tuple_windows())
            .filter_map(|element| match element {
                Both(&waypoint, (before, after)) => {
                    if before.angle(&after) > spike_angle {
                        None
                    } else {
                        Some(waypoint)
                    }
                }
                Left(&waypoint) => Some(waypoint),
                Right(_) => unreachable!(),
            })
            .dedup();

        once(shape[0]).chain(cleaned).collect()
    }

    fn smooth_zigzags(shape: Shape) -> Shape {
        let zigzag_angle = 20_f64.to_radians();
        let segments = shape.windows(2).map(|adjacent| adjacent[1] - adjacent[0]);

        let mut was_merged = false;
        let smoothed = shape
            .iter()
            .skip(1)
            .zip_longest(segments.tuple_windows())
            .filter_map(|element| match element {
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
                }
                Left(&waypoint) => Some(waypoint),
                Right(_) => unreachable!(),
            });

        once(shape[0]).chain(smoothed).collect()
    }
}

#[cfg(test)]
mod tests {
    use na::Vector2;

    use super::*;
    use crate::coord::project;
    use crate::{dataset, map, shape};

    #[test]
    fn test_remove_overlapping() {
        let mut shape = shape!(blue);
        shape.insert(3, shape[2]);
        assert_eq!(Importer::smooth(shape), shape!(blue));
    }

    #[test]
    fn test_remove_spike() {
        let mut shape = shape!(blue);
        shape.insert(3, project(13.392, 52.508));
        assert_eq!(Importer::smooth(shape), shape!(blue));
    }

    #[test]
    fn test_remove_jump() {
        let mut shape = shape!(blue);
        shape.insert(3, project(13.386, 52.521));
        shape.insert(4, shape[2]);
        assert_eq!(Importer::smooth(shape), shape!(blue));
    }

    #[test]
    fn test_smooth_zigzag() {
        let mut shape = shape!(blue);
        let original = shape.remove(2);
        let offset = Vector2::new(0.0, 100.0);
        shape.insert(2, original + offset);
        shape.insert(3, original - offset);
        assert_eq!(Importer::smooth(shape), shape!(blue));
    }

    #[test]
    fn test_from_csv() {
        let mut dataset = dataset!(
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

        assert_eq!(
            Importer::import(&mut dataset).unwrap(),
            map! {
                "1" => shape!(52.51, 13.37; 52.52, 13.37),
                "2" => shape!(blue),
            }
        );
    }
}
