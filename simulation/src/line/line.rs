use itertools::Itertools;

use na::Vector2;

use super::Kind;
use crate::color::Color;
use crate::direction::Direction;
use crate::node::Node;
use crate::train::Train;

#[derive(Debug, PartialEq)]
pub struct Line {
    name: String,
    color: Color,
    kind: Kind,
    nodes: Vec<Node>,
    trains: Vec<Train>,
}

impl Line {
    pub fn new(
        name: String,
        color: Color,
        kind: Kind,
        nodes: Vec<Node>,
        trains: Vec<Train>,
    ) -> Line {
        Line {
            name,
            color,
            kind,
            nodes,
            trains,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn fill_color_buffer(&self, colors: &mut Vec<f32>) {
        colors.extend(self.color.iter().map(|component| component as f32 / 255.0));
    }

    pub fn active_trains(&self) -> impl Iterator<Item = &Train> {
        self.trains.iter().filter(|train| train.is_active())
    }

    pub fn update(&mut self, time_passed: u32) {
        for train in &mut self.trains {
            train.update(time_passed, &self.nodes);
        }
    }

    pub fn fill_vertices_buffer_with_lengths(
        &self,
        vertices: &mut Vec<f32>,
        lengths: &mut Vec<usize>,
    ) {
        self.fill_vertices_buffer_with_lengths_for_direction(
            Direction::Upstream,
            vertices,
            lengths,
        );
        self.fill_vertices_buffer_with_lengths_for_direction(
            Direction::Downstream,
            vertices,
            lengths,
        );
    }

    fn fill_vertices_buffer_with_lengths_for_direction(
        &self,
        direction: Direction,
        vertices: &mut Vec<f32>,
        lengths: &mut Vec<usize>,
    ) {
        let length = vertices.len();
        self.fill_vertices_buffer_for_direction(direction, vertices);
        lengths.push((vertices.len() - length) / 2);
    }

    fn fill_vertices_buffer_for_direction(&self, direction: Direction, vertices: &mut Vec<f32>) {
        let nodes = self.nodes.iter().filter(|node| node.allows(direction));

        let mut segments = nodes
            .clone()
            .tuple_windows()
            .map(|(before, after)| after.position() - before.position())
            .collect::<Vec<_>>();
        if segments.len() == 0 {
            return;
        }
        segments.insert(0, segments.first().unwrap().clone());
        segments.insert(segments.len(), segments.last().unwrap().clone());

        for (node, adjacent) in nodes.zip_eq(segments.windows(2)) {
            let perp = adjacent[0].perp(&adjacent[1]);
            let miter = if perp == 0.0 {
                Vector2::new(-adjacent[0].y, adjacent[0].x).normalize()
            } else {
                let preceding = adjacent[0] * adjacent[1].norm();
                let following = adjacent[1] * adjacent[0].norm();
                (following - preceding) / perp
            };

            self.add_node_vertices_to_buffer(node, miter, vertices);
        }
    }

    fn add_node_vertices_to_buffer(
        &self,
        node: &Node,
        mut miter: Vector2<f32>,
        vertices: &mut Vec<f32>,
    ) {
        miter *= self.kind.line_width() * 0.5;
        vertices.extend((node.position() + miter).iter());
        vertices.extend((node.position() - miter).iter());
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use super::*;
    use crate::fixtures::{nodes, trains};

    macro_rules! lines {
        (trains $line:ident, $route:ident, [$( $hour:expr, $minute:expr );* $(;)?]) => {
            $( trains::$line::$route($hour, $minute) ),*
        };
        ($($line:ident: $name:literal, $kind:ident, $upstream:ident, $upstream_times:tt, $downstream:ident, $downstream_times:tt);* $(;)?) => {
            $(
                pub fn $line() -> Line {
                    Line {
                        name: $name.to_string(),
                        color: Kind::$kind.color(),
                        kind: Kind::$kind,
                        nodes: nodes::$line(),
                        trains: vec![
                            lines!(trains $line, $upstream, $upstream_times),
                            lines!(trains $line, $downstream, $downstream_times),
                        ],
                    }
                }
            )*
        };
    }

    lines! {
        tram_12:            "12",           Tram,
            oranienburger_tor_am_kupfergraben, [9, 2.0],
            am_kupfergraben_oranienburger_tor, [8, 34.0];
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use na::Point2;

    use super::*;
    use crate::direction::Directions;
    use crate::fixtures::{lines, nodes};
    use crate::node::Kind as NodeKind;

    fn time(hour: u32, minute: f64) -> u32 {
        return hour * 3600 + (minute * 60.0) as u32;
    }

    #[test]
    fn test_getters() {
        let line = lines::tram_12();
        assert_eq!(line.name(), "12");
        assert_eq!(line.kind(), Kind::Tram);
        assert_eq!(line.nodes(), &*nodes::tram_12());
    }

    #[test]
    fn test_fill_color_buffer() {
        let line = lines::tram_12();
        let mut colors = Vec::new();
        line.fill_color_buffer(&mut colors);
        assert_relative_eq!(*colors, [0.8, 0.04, 0.13], epsilon = 0.01);
    }

    #[test]
    fn test_active_trains() {
        let mut line = lines::tram_12();
        assert_eq!(line.active_trains().count(), 0);
        line.update(time(8, 34.0));
        assert_eq!(line.active_trains().count(), 0);
        line.update(time(0, 1.0));
        assert_eq!(line.active_trains().count(), 1);
        line.update(time(0, 6.0));
        assert_eq!(line.active_trains().count(), 0);
        line.update(time(0, 22.0));
        assert_eq!(line.active_trains().count(), 1);
    }

    macro_rules! test_vertices {
        ($nodes:tt, $upstream:tt) => (
            test_vertices!($nodes, $upstream, $upstream)
        );
        ($nodes:tt, $kind:ident, $upstream:tt) => (
            test_vertices!($nodes, $kind, $upstream, $upstream);
        );
        ($nodes:tt, $upstream:tt, $downstream:tt) => (
            test_vertices!($nodes, Railway, $upstream, $downstream)
        );
        (
            [$($x:literal, $y:literal, $in_directions:ident);* $(;)?],
            $kind:ident,
            [$($upstream:literal),* $(,)?],
            [$($downstream:literal),* $(,)?] $(,)?
        ) => (
            let line = Line {
                name: String::new(),
                color: Kind::$kind.color(),
                kind: Kind::$kind,
                nodes: vec![ $(
                    Node::new(Point2::new($x, $y), NodeKind::Waypoint, Directions::$in_directions)
                ),* ],
                trains: Vec::new(),
            };
            let mut upstream_vertices = Vec::new();
            line.fill_vertices_buffer_for_direction(Direction::Upstream, &mut upstream_vertices);
            assert_relative_eq!(
                *upstream_vertices,
                [ $( $upstream ),* ]
            );
            let mut downstream_vertices = Vec::new();
            line.fill_vertices_buffer_for_direction(Direction::Downstream, &mut downstream_vertices);
            assert_relative_eq!(
                *downstream_vertices,
                [ $( $downstream ),* ]
            );
            let mut vertices = Vec::new();
            let mut lengths = Vec::new();
            line.fill_vertices_buffer_with_lengths(&mut vertices, &mut lengths);
            assert_eq!(lengths, [upstream_vertices.len() / 2, downstream_vertices.len() / 2]);
            upstream_vertices.append(&mut downstream_vertices);
            assert_eq!(vertices, upstream_vertices);
        );
    }

    #[test]
    fn test_empty_vertices() {
        test_vertices!([], []);
    }

    #[test]
    fn test_straight_vertices() {
        test_vertices!([
               0.0,    0.0, Both;
             100.0,    0.0, Both;
             200.0,    0.0, Both;
        ], [
               0.0,   25.0,    0.0,  -25.0,
             100.0,   25.0,  100.0,  -25.0,
             200.0,   25.0,  200.0,  -25.0,
        ]);
    }

    #[test]
    fn test_different_line_size() {
        test_vertices!([
               0.0,    0.0, Both;
             100.0,    0.0, Both;
             200.0,    0.0, Both;
        ],
        SuburbanRailway,
        [
               0.0,   20.0,    0.0,  -20.0,
             100.0,   20.0,  100.0,  -20.0,
             200.0,   20.0,  200.0,  -20.0,
        ]);
    }

    #[test]
    fn test_right_angle_vertices() {
        test_vertices!([
               0.0,    0.0, Both;
             100.0,    0.0, Both;
             100.0,  100.0, Both;
        ], [
               0.0,   25.0,    0.0,  -25.0,
              75.0,   25.0,  125.0,  -25.0,
              75.0,  100.0,  125.0,  100.0,
        ]);
    }

    #[test]
    fn test_different_direction_vertices() {
        test_vertices!([
                0.0,    0.0, Both;
              100.0,    0.0, UpstreamOnly;
                0.0,  100.0, DownstreamOnly;
              100.0,  100.0, Both;
        ], [
               0.0,   25.0,    0.0,  -25.0,
              75.0,   25.0,  125.0,  -25.0,
              75.0,  100.0,  125.0,  100.0,
        ], [
             -25.0,    0.0,   25.0,    0.0,
             -25.0,  125.0,   25.0,   75.0,
             100.0,  125.0,  100.0,   75.0,
        ]);
    }
}
