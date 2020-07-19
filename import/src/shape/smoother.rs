use std::error::Error;
use std::fmt;
use std::str::FromStr;

use crate::coord::Point;
use crate::shape::Shape;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    Off,
    Deduplicate,
    Full,
}

impl Mode {
    pub(super) fn smooth(self, shape: Shape) -> Shape {
        let mut smoother = Smoother::new(self);
        for position in shape {
            smoother.add(position);
        }
        smoother.finish()
    }

    pub fn variants() -> &'static [&'static str] {
        &["off", "deduplicate", "full"]
    }
}

#[derive(Debug, Clone)]
pub struct InvalidModeError(String);

impl fmt::Display for InvalidModeError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "smoothing mode '{}' not found", self.0)
    }
}

impl Error for InvalidModeError {}

impl FromStr for Mode {
    type Err = InvalidModeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "off" => Ok(Self::Off),
            "deduplicate" => Ok(Self::Deduplicate),
            "full" => Ok(Self::Full),
            _ => Err(InvalidModeError(value.to_string())),
        }
    }
}

#[derive(Debug)]
struct Smoother {
    mode: Mode,
    points: Vec<Point>,
}

impl Smoother {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            points: Vec::new(),
        }
    }

    fn dedup(&mut self) -> bool {
        let len = self.points.len();
        if len >= 2 && self.points[len - 2] == self.points[len - 1] {
            self.points.pop();
            return true;
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

        match self.mode {
            Mode::Off => {}
            Mode::Deduplicate => {
                self.dedup();
            }
            Mode::Full => {
                if self.dedup() {
                    return;
                }
                if self.remove_spike() {
                    self.dedup();
                }
                while self.smooth_zigzag() {}
            }
        }
    }

    fn finish(self) -> Shape {
        Shape::from(self.points)
    }
}

#[cfg(test)]
mod tests {
    use na::Vector2;

    use super::*;
    use crate::coord::project;
    use crate::shape;
    use test_utils::assert_eq_alternate;

    #[test]
    fn test_remove_overlapping() {
        let mut shape = shape!(blue);
        shape.insert(3, shape[2]);
        assert_eq_alternate!(
            Mode::Deduplicate.smooth(Shape::from(shape)),
            Shape::from(shape!(blue))
        );
    }

    #[test]
    fn test_remove_spike() {
        let mut shape = shape!(blue);
        shape.insert(3, project(13.392, 52.508));
        assert_eq_alternate!(
            Mode::Full.smooth(Shape::from(shape)),
            Shape::from(shape!(blue))
        );
    }

    #[test]
    fn test_remove_jump() {
        let mut shape = shape!(blue);
        shape.insert(3, project(13.386, 52.521));
        shape.insert(4, shape[2]);
        assert_eq_alternate!(
            Mode::Full.smooth(Shape::from(shape)),
            Shape::from(shape!(blue))
        );
    }

    #[test]
    fn test_smooth_zigzag() {
        let mut shape = shape!(blue);
        let original = shape.remove(2);
        let offset = Vector2::new(0.0, 100.0);
        shape.insert(2, original + offset);
        shape.insert(3, original - offset);
        assert_eq_alternate!(
            Mode::Full.smooth(Shape::from(shape)),
            Shape::from(shape!(blue))
        );
    }
}
