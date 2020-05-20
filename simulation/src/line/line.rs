use itertools::Itertools;

use na::Vector2;

use super::Kind;
use crate::direction::Direction;
use crate::node::Node;
use crate::train::Train;

#[derive(Debug)]
pub struct Line {
    name: String,
    kind: Kind,
    nodes: Vec<Node>,
    trains: Vec<Train>,
}

impl Line {
    pub fn new(name: String, kind: Kind, nodes: Vec<Node>, trains: Vec<Train>) -> Line {
        Line {
            name,
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

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use na::Point2;

    use super::*;
    use crate::direction::Directions;
    use crate::node::Kind as NodeKind;

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
