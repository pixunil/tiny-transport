use serde_derive::{Serialize, Deserialize};
use na::{Point2, Vector2, Matrix2};

use crate::line::LineNode;

#[derive(Debug, PartialEq, Clone, Copy)]
enum TrainState {
    WaitingForDispatch,
    Stopped {
        at: usize,
    },
    Driving {
        from: usize,
        to: usize,
    },
    Finished,
}

impl TrainState {
    fn next(self, direction: Direction, nodes: &[LineNode]) -> TrainState {
        let (at, already_stopped) = match self {
            TrainState::WaitingForDispatch => {
                (direction.start(nodes.len()), false)
            },
            TrainState::Driving { from: _, to } => {
                (to, false)
            },
            TrainState::Stopped { at } => {
                (at, true)
            },
            TrainState::Finished => return TrainState::Finished,
        };

        if !already_stopped && nodes[at].is_stop() {
            TrainState::Stopped { at }
        } else if at == direction.end(nodes.len()) {
            TrainState::Finished
        } else {
            TrainState::Driving { from: at, to: direction.next(at) }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Upstream,
    Downstream,
}

impl Direction {
    fn start(self, len: usize) -> usize {
        match self {
            Direction::Upstream => 0,
            Direction::Downstream => len - 1,
        }
    }

    fn end(self, len: usize) -> usize {
        match self {
            Direction::Upstream => len - 1,
            Direction::Downstream => 0,
        }
    }

    fn next(self, position: usize) -> usize {
        match self {
            Direction::Upstream => position + 1,
            Direction::Downstream => position - 1,
        }
    }
}

#[derive(Debug)]
pub struct Train {
    direction: Direction,
    durations: Vec<u32>,
    current: usize,
    current_passed: u32,
    state: TrainState,
}

impl Train {
    pub fn new(direction: Direction, durations: Vec<u32>) -> Train {
        Train {
            direction,
            durations,
            current: 0,
            current_passed: 0,
            state: TrainState::WaitingForDispatch,
        }
    }

    pub fn update(&mut self, time_passed: u32, nodes: &[LineNode]) {
        self.current_passed += time_passed;

        while self.current < self.durations.len() && self.current_passed > self.durations[self.current] {
            self.current_passed -= self.durations[self.current];
            self.current += 1;
            self.state = self.state.next(self.direction, nodes);
        }
    }

    pub fn is_active(&self) -> bool {
        match self.state {
            TrainState::Driving { .. } | TrainState::Stopped { .. } => true,
            TrainState::WaitingForDispatch | TrainState::Finished => false,
        }
    }

    pub fn fill_vertice_buffer(&self, buffer: &mut Vec<f32>, nodes: &[LineNode]) {
        let (position, orientation) = self.calculate_rectangle(nodes);
        self.write_rectangle(buffer, position, orientation);
    }

    fn calculate_rectangle(&self, nodes: &[LineNode]) -> (Point2<f32>, Vector2<f32>) {
        match self.state {
            TrainState::Stopped { at } => {
                let current = nodes[at].position();
                let orientation = if at == 0 {
                    nodes[at + 1].position() - current
                } else if at == nodes.len() - 1 {
                    current - nodes[at - 1].position()
                } else {
                    nodes[at + 1].position() - nodes[at - 1].position()
                };
                (current, orientation.normalize())
            },
            TrainState::Driving { from, to } => {
                let travelled = self.current_passed as f32 / self.durations[self.current] as f32;
                let from = nodes[from].position();
                let to = nodes[to].position();
                let segment = to - from;
                (from + segment * travelled, segment.normalize())
            },
            TrainState::WaitingForDispatch | TrainState::Finished => unreachable!(),
        }
    }

    fn write_rectangle(&self, buffer: &mut Vec<f32>, position: Point2<f32>, orientation: Vector2<f32>) {
        let bounds = Matrix2::from_columns(&[orientation, Vector2::new(-orientation.y, orientation.x)]);
        let right_front = position + bounds * Vector2::new(4.5, 3.0);
        let left_front = position + bounds * Vector2::new(4.5, -3.0);
        let right_back = position + bounds * Vector2::new(-4.5, 3.0);
        let left_back = position + bounds * Vector2::new(-4.5, -3.0);
        buffer.extend(left_back.iter().chain(left_front.iter()).chain(right_back.iter()));
        buffer.extend(right_front.iter().chain(right_back.iter()).chain(left_front.iter()));
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    use crate::line_nodes;

    fn train() -> Train {
        Train::new(Direction::Upstream, vec![10, 1, 2, 2, 1])
    }

    #[test]
    fn test_before_dispatch() {
        let mut train = train();
        train.update(0, &line_nodes!(blue));
        assert_eq!(train.state, TrainState::WaitingForDispatch);
        assert!(!train.is_active());
    }

    #[test]
    fn test_stopped() {
        let mut train = train();
        train.update(11, &line_nodes!(blue));
        assert_eq!(train.state, TrainState::Stopped { at: 0 });
        assert!(train.is_active());

        let (position, orientation) = train.calculate_rectangle(&line_nodes!(blue));
        assert_relative_eq!(position, Point2::new(200.0, 100.0));
        assert_relative_eq!(orientation, Vector2::new(1.0, 0.0));
    }

    #[test]
    fn test_driving() {
        let mut train = train();
        train.update(12, &line_nodes!(blue));
        assert_eq!(train.state, TrainState::Driving { from: 0, to: 1 });
        assert!(train.is_active());

        let (position, orientation) = train.calculate_rectangle(&line_nodes!(blue));
        assert_relative_eq!(position, Point2::new(210.0, 100.0));
        assert_relative_eq!(orientation, Vector2::new(1.0, 0.0));
    }

    #[test]
    fn test_finished() {
        let mut train = train();
        train.update(17, &line_nodes!(blue));
        assert_eq!(train.state, TrainState::Finished);
        assert!(!train.is_active());
    }

    #[test]
    fn test_nodes_before_start() {
        let mut nodes = line_nodes!(blue);
        nodes.insert(0, LineNode::new(Point2::new(190.0, 100.0)));
        let mut train = train();
        train.durations.insert(1, 1);

        train.update(11, &nodes);
        assert_eq!(train.state, TrainState::Driving { from: 0, to: 1 });
        assert!(train.is_active());
    }

    #[test]
    fn test_nodes_after_terminus() {
        let mut nodes = line_nodes!(blue);
        nodes.push(LineNode::new(Point2::new(240.0, 105.0)));
        let mut train = train();
        train.durations.push(1);

        train.update(17, &nodes);
        assert_eq!(train.state, TrainState::Driving { from: 2, to: 3 });
        assert!(train.is_active());
    }

    #[test]
    fn test_rectangle_horizontal() {
        let mut train = train();
        train.update(10, &line_nodes!(blue));
        let mut buffer = Vec::new();
        train.write_rectangle(&mut buffer, Point2::new(200.0, 100.0), Vector2::new(1.0, 0.0));
        assert_relative_eq!(*buffer, [
            195.5, 97.0,
            204.5, 97.0,
            195.5, 103.0,
            204.5, 103.0,
            195.5, 103.0,
            204.5, 97.0,
        ]);
    }
}
