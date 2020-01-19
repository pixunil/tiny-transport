use itertools::Itertools;

use na::Vector2;

use crate::direction::Direction;
use crate::node::Node;
use crate::train::Train;
use super::Kind;

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

    pub(super) fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub(super) fn active_trains(&self) -> impl Iterator<Item = &Train> {
        self.trains.iter()
            .filter(|train| train.is_active())
    }

    pub fn update(&mut self, time_passed: u32) {
        for train in &mut self.trains {
            train.update(time_passed, &self.nodes);
        }
    }

    pub(super) fn fill_vertices_buffer_with_lengths(&self, vertices: &mut Vec<f32>, lengths: &mut Vec<usize>) {
        self.fill_vertices_buffer_with_lengths_for_direction(Direction::Upstream, vertices, lengths);
        self.fill_vertices_buffer_with_lengths_for_direction(Direction::Downstream, vertices, lengths);
    }

    fn fill_vertices_buffer_with_lengths_for_direction(&self, direction: Direction, vertices: &mut Vec<f32>, lengths: &mut Vec<usize>) {
        let length = vertices.len();
        self.fill_vertices_buffer_for_direction(direction, vertices);
        lengths.push((vertices.len() - length) / 2);
    }

    fn fill_vertices_buffer_for_direction(&self, direction: Direction, vertices: &mut Vec<f32>) {
        let nodes = self.nodes.iter()
            .filter(|node| node.allows(direction));

        let mut segments = nodes.clone()
            .tuple_windows()
            .map(|(before, after)| after.position() - before.position())
            .collect::<Vec<_>>();
        if segments.len() == 0 {
            return;
        }
        segments.insert(0, segments.first().unwrap().clone());
        segments.insert(segments.len(), segments.last().unwrap().clone());

        for (waypoint, adjacent) in nodes.zip_eq(segments.windows(2)) {
            let perp = adjacent[0].perp(&adjacent[1]);
            let miter = if perp == 0.0 {
                Vector2::new(-adjacent[0].y, adjacent[0].x).normalize()
            } else {
                let preceding = adjacent[0] * adjacent[1].norm();
                let following = adjacent[1] * adjacent[0].norm();
                (following - preceding) / perp
            };

            vertices.extend((waypoint.position() + miter).iter());
            vertices.extend((waypoint.position() - miter).iter());
        }
    }
}

#[cfg(test)]
mod fixtures {
    use super::*;

    use crate::node::fixtures as nodes;

    pub(super) fn blue() -> Line {
        Line {
            name: "Blue Line".to_string(),
            kind: Kind::SuburbanRailway,
            nodes: nodes::blue(),
            trains: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    use super::fixtures as lines;

    fn blue_line_vertices() -> Vec<f32> {
        vec![
            200.0, 101.0,
            200.0, 99.0,
            219.76, 101.0,
            220.24, 99.0,
            229.55, 105.89,
            230.45, 104.11,
        ]
    }

    #[test]
    fn test_upstream_vertices() {
        let line = lines::blue();
        let mut vertices = Vec::new();
        line.fill_vertices_buffer_for_direction(Direction::Upstream, &mut vertices);
        assert_relative_eq!(*vertices, blue_line_vertices(), epsilon = 0.01);
    }

    #[test]
    fn test_length_buffer() {
        let line = lines::blue();
        let mut vertices = Vec::new();
        let mut lengths = Vec::new();
        line.fill_vertices_buffer_with_lengths(&mut vertices, &mut lengths);
        assert_eq!(lengths, [6, 6]);
    }
}
