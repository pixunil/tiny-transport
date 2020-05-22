use na::{Matrix2, Point2, Vector2};

use crate::direction::Direction;
use crate::line::Kind;
use crate::node::Node;

#[derive(Debug, PartialEq, Clone, Copy)]
enum TrainState {
    WaitingForDispatch,
    Stopped { at: usize },
    Driving { from: usize, to: usize },
    Finished,
}

impl TrainState {
    fn next(self, direction: Direction, nodes: &[Node]) -> TrainState {
        let (at, already_stopped) = match self {
            TrainState::WaitingForDispatch => (direction.start(nodes.len()), false),
            TrainState::Driving { from: _, to } => (to, false),
            TrainState::Stopped { at } => (at, true),
            TrainState::Finished => return TrainState::Finished,
        };

        if !already_stopped && nodes[at].is_stop() {
            TrainState::Stopped { at }
        } else if let Some(to) = direction.find_next(at, nodes) {
            TrainState::Driving { from: at, to }
        } else {
            TrainState::Finished
        }
    }
}

#[derive(Debug)]
pub struct Train {
    kind: Kind,
    direction: Direction,
    durations: Vec<u32>,
    current: usize,
    current_passed: u32,
    state: TrainState,
}

impl Train {
    pub fn new(kind: Kind, direction: Direction, durations: Vec<u32>) -> Train {
        Train {
            kind,
            direction,
            durations,
            current: 0,
            current_passed: 0,
            state: TrainState::WaitingForDispatch,
        }
    }

    pub fn update(&mut self, time_passed: u32, nodes: &[Node]) {
        self.current_passed += time_passed;

        while self.current < self.durations.len()
            && self.current_passed > self.durations[self.current]
        {
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

    pub fn fill_vertice_buffer(&self, buffer: &mut Vec<f32>, nodes: &[Node]) {
        let (position, orientation) = self.calculate_rectangle(nodes);
        self.write_rectangle(buffer, position, orientation);
    }

    fn calculate_rectangle(&self, nodes: &[Node]) -> (Point2<f32>, Vector2<f32>) {
        match self.state {
            TrainState::Stopped { at } => {
                let current = nodes[at].position();
                let previous = self.direction.find_previous(at, nodes).unwrap_or(at);
                let next = self.direction.find_next(at, nodes).unwrap_or(at);
                let orientation = nodes[next].position() - nodes[previous].position();
                (current, orientation.normalize())
            }
            TrainState::Driving { from, to } => {
                let travelled = self.current_passed as f32 / self.durations[self.current] as f32;
                let from = nodes[from].position();
                let to = nodes[to].position();
                let segment = to - from;
                (from + segment * travelled, segment.normalize())
            }
            TrainState::WaitingForDispatch | TrainState::Finished => unreachable!(),
        }
    }

    fn write_rectangle(
        &self,
        buffer: &mut Vec<f32>,
        position: Point2<f32>,
        orientation: Vector2<f32>,
    ) {
        let bounds =
            Matrix2::from_columns(&[orientation, Vector2::new(-orientation.y, orientation.x)]);
        let right_front =
            position + bounds * Vector2::new(0.5, 0.5).component_mul(&self.kind.train_size());
        let left_front =
            position + bounds * Vector2::new(0.5, -0.5).component_mul(&self.kind.train_size());
        let right_back =
            position + bounds * Vector2::new(-0.5, 0.5).component_mul(&self.kind.train_size());
        let left_back =
            position + bounds * Vector2::new(-0.5, -0.5).component_mul(&self.kind.train_size());
        buffer.extend(
            left_back
                .iter()
                .chain(left_front.iter())
                .chain(right_back.iter()),
        );
        buffer.extend(
            right_front
                .iter()
                .chain(right_back.iter())
                .chain(left_front.iter()),
        );
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    macro_rules! trains {
        ($($train:ident : $kind:ident, { $( $route:ident => $direction:ident, [$($time:literal),* $(,)?]);* $(;)? } ),* $(,)?) => (
            $(
                pub mod $train {
                    use crate::train::*;
                    $(
                        pub fn $route(hour: u32, minute: f64) -> Train {
                            Train::new(
                                Kind::$kind,
                                Direction::$direction,
                                vec![
                                    hour * 3600 + (minute * 60.0) as u32,
                                    $($time),*
                                ],
                            )
                        }
                    )*
                }
            )*
        );
    }

    trains! {
        tram_m5: Tram, {
            zingster_str_perower_platz => Upstream, [0, 0, 60, 0, 60, 0, 48, 72, 0];
        },
        tram_12: Tram, {
            oranienburger_tor_am_kupfergraben =>
                Upstream, [0, 24, 72, 24, 0, 35, 21, 21, 21, 21, 0, 60, 0];
            am_kupfergraben_oranienburger_tor =>
                Downstream, [0, 15, 18, 9, 18, 0, 48, 24, 39, 24, 46, 0, 24, 24, 48, 24, 0];
        },
        bus_m82: Bus, {
            weskammstr_waldsassener_str => Upstream, [0, 0, 15, 15, 0, 7, 7, 8, 7, 0, 0, 0, 0];
        },
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::fixtures::*;

    fn time(hour: u32, minute: f64) -> u32 {
        return hour * 3600 + (minute * 60.0) as u32;
    }

    fn segment_vector(nodes: &[Node], from: usize, to: usize) -> Vector2<f32> {
        (nodes[to].position() - nodes[from].position()).normalize()
    }

    #[test]
    fn test_before_dispatch() {
        let mut train = trains::tram_12::oranienburger_tor_am_kupfergraben(9, 2.0);
        train.update(0, &nodes::tram_12());
        assert_eq!(train.state, TrainState::WaitingForDispatch);
        assert!(!train.is_active());
    }

    #[test]
    fn test_stopped() {
        let mut train = trains::tram_12::oranienburger_tor_am_kupfergraben(9, 2.0);
        train.durations[1] = 30;
        train.update(time(9, 2.5), &nodes::tram_12());
        assert_eq!(train.state, TrainState::Stopped { at: 0 });
        assert!(train.is_active());

        let (position, orientation) = train.calculate_rectangle(&nodes::tram_12());
        assert_relative_eq!(position, Point2::new(-98.0, -1671.0));
        assert_relative_eq!(orientation, segment_vector(&nodes::tram_12(), 0, 1));
    }

    #[test]
    fn test_driving() {
        let mut train = trains::tram_12::oranienburger_tor_am_kupfergraben(9, 2.0);
        train.update(time(9, 3.0), &nodes::tram_12());
        assert_eq!(train.state, TrainState::Driving { from: 1, to: 3 });
        assert!(train.is_active());

        let (position, orientation) = train.calculate_rectangle(&nodes::tram_12());
        assert_relative_eq!(position, Point2::new(-104.5, -1393.0));
        assert_relative_eq!(orientation, segment_vector(&nodes::tram_12(), 1, 3));
    }

    #[test]
    fn test_upstream_ignores_downstream_only() {
        let mut train = trains::tram_12::oranienburger_tor_am_kupfergraben(9, 2.0);
        train.update(time(9, 6.5), &nodes::tram_12());
        assert_eq!(train.state, TrainState::Driving { from: 9, to: 16 });
    }

    #[test]
    fn test_downstream_ignores_upstream_only() {
        let mut train = trains::tram_12::am_kupfergraben_oranienburger_tor(8, 34.0);
        train.update(time(8, 36.5), &nodes::tram_12());
        assert_eq!(train.state, TrainState::Driving { from: 10, to: 7 });
    }

    #[test]
    fn test_finished() {
        let mut train = trains::tram_12::oranienburger_tor_am_kupfergraben(9, 2.0);
        train.update(time(9, 7.0), &nodes::tram_12());
        assert_eq!(train.state, TrainState::Finished);
        assert!(!train.is_active());
    }

    #[test]
    fn test_nodes_before_start() {
        let mut train = trains::tram_m5::zingster_str_perower_platz(8, 13.0);
        train.durations[1] = 30;
        train.update(time(8, 13.5), &nodes::tram_m5());
        assert_eq!(train.state, TrainState::Driving { from: 0, to: 1 });
        assert!(train.is_active());
    }

    #[test]
    fn test_nodes_after_terminus() {
        let mut train = trains::bus_m82::weskammstr_waldsassener_str(9, 46.0);
        train.durations[13] = 30;
        train.update(time(9, 47.0), &nodes::bus_m82());
        assert_eq!(train.state, TrainState::Driving { from: 10, to: 11 });
        assert!(train.is_active());
    }

    #[test]
    fn test_rectangle_horizontal() {
        let train = Train::new(Kind::SuburbanRailway, Direction::Upstream, Vec::new());
        let mut buffer = Vec::new();
        train.write_rectangle(
            &mut buffer,
            Point2::new(250.0, 200.0),
            Vector2::new(1.0, 0.0),
        );
        assert_relative_eq!(
            *buffer,
            [140.0, 125.0, 360.0, 125.0, 140.0, 275.0, 360.0, 275.0, 140.0, 275.0, 360.0, 125.0,]
        );
    }
}
