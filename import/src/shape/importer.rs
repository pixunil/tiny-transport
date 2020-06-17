use std::collections::HashMap;
use std::error::Error;

use super::{Shape, ShapeId, ShapeRecord};
use crate::coord::Point;
use crate::utils::Action;
use crate::utils::Dataset;

pub(crate) struct Importer;

impl Importer {
    pub(crate) fn import(
        dataset: &mut impl Dataset,
    ) -> Result<HashMap<ShapeId, Shape>, Box<dyn Error>> {
        let mut shapes = HashMap::new();

        let action = Action::start("Importing shapes");
        for result in action.read_csv(dataset, "shapes.txt")? {
            let record: ShapeRecord = result?;
            record.import(&mut shapes);
        }
        action.complete(&format!("Imported {} shapes", shapes.len()));

        let mut action = Action::start("Smoothing shapes");
        let shapes = action
            .wrap_iter(shapes)
            .map(|(id, shape)| (id, Self::smooth(shape)))
            .collect();
        action.complete("Smoothed shapes");
        Ok(shapes)
    }

    fn smooth(shape: Shape) -> Shape {
        let mut smoother = PointSmoother::new();
        for point in shape {
            smoother.add(point);
        }
        smoother.points
    }
}

struct PointSmoother {
    points: Vec<Point>,
}

impl PointSmoother {
    fn new() -> Self {
        Self { points: Vec::new() }
    }

    fn dedup(&mut self) -> bool {
        let len = self.points.len();
        if len >= 2 {
            if self.points[len - 2] == self.points[len - 1] {
                self.points.pop();
                return true;
            }
        }
        false
    }

    fn remove_spike(&mut self) -> bool {
        let len = self.points.len();
        if len >= 3 {
            let spike_angle = 120_f64.to_radians();
            let before = self.points[len - 2] - self.points[len - 3];
            let after = self.points[len - 1] - self.points[len - 2];
            if before.angle(&after) > spike_angle {
                self.points.remove(len - 2);
                return true;
            }
        }
        false
    }

    fn smooth_zigzag(&mut self) -> bool {
        let len = self.points.len();
        if len >= 4 {
            let zigzag_angle = 20_f64.to_radians();
            let before = self.points[len - 3] - self.points[len - 4];
            let between = self.points[len - 2] - self.points[len - 3];
            let after = self.points[len - 1] - self.points[len - 2];
            let angles = [
                before.angle(&between),
                between.angle(&after),
                before.angle(&after),
            ];

            if (angles[0] + angles[1] - angles[2]).abs() > zigzag_angle {
                self.points[len - 3] += between / 2.0;
                self.points.remove(len - 2);
                return true;
            }
        }
        false
    }

    fn add(&mut self, point: Point) {
        self.points.push(point);

        if self.dedup() {
            return;
        }
        if self.remove_spike() {
            self.dedup();
        }
        while self.smooth_zigzag() {}
    }
}

#[cfg(test)]
mod tests {
    use na::Vector2;

    use super::*;
    use crate::coord::project;
    use crate::{dataset, shape};
    use test_utils::map;

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
