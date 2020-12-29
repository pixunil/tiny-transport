use itertools::Itertools;

use na::Vector2;

use super::Kind;
use crate::color::Color;
use crate::path::{Node, Segment, SegmentedPath};
use crate::train::Train;

#[derive(Debug, PartialEq)]
pub struct Line {
    name: String,
    color: Color,
    kind: Kind,
    path: SegmentedPath,
    trains: Vec<Train>,
}

impl Line {
    pub fn new(
        name: String,
        color: Color,
        kind: Kind,
        path: SegmentedPath,
        trains: Vec<Train>,
    ) -> Line {
        Line {
            name,
            color,
            kind,
            path,
            trains,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn path(&self) -> &SegmentedPath {
        &self.path
    }

    pub fn fill_color_buffer(&self, colors: &mut Vec<f32>) {
        colors.extend(self.color.iter().map(|component| component as f32 / 255.0));
    }

    pub fn active_trains(&self) -> impl Iterator<Item = &Train> {
        self.trains.iter().filter(|train| train.is_active())
    }

    pub fn update(&mut self, segments: &[Segment], time_passed: u32) {
        let nodes = self.path.nodes(segments);
        for train in &mut self.trains {
            train.update(time_passed, &nodes);
        }
    }

    pub fn fill_vertices_buffer_with_lengths(
        &self,
        segments: &[Segment],
        vertices: &mut Vec<f32>,
        lengths: &mut Vec<usize>,
    ) {
        let length = vertices.len();
        self.fill_vertices_buffer(segments, vertices);
        lengths.push((vertices.len() - length) / 2);
    }

    fn fill_vertices_buffer(&self, segments: &[Segment], vertices: &mut Vec<f32>) {
        let nodes = self.path.nodes(segments);

        let mut segments = nodes
            .iter()
            .tuple_windows()
            .map(|(before, after)| after.position() - before.position())
            .collect::<Vec<_>>();
        if segments.is_empty() {
            return;
        }
        segments.insert(0, *segments.first().unwrap());
        segments.insert(segments.len(), *segments.last().unwrap());

        for (node, adjacent) in nodes.iter().zip_eq(segments.windows(2)) {
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
    use std::ops::Index;

    use super::*;
    use crate::fixtures::{paths, trains};
    use common::time;

    macro_rules! lines {
        (@trains $line:ident, $route:ident, [ $( $( $(:)? $time:literal )* ),* ]) => {
            $( trains::$line::$route(time!($($time),*)) ),*
        };
        ($($line:ident: $name:literal, $kind:ident, $route:ident, $times:tt);* $(;)?) => {
            $(
                pub fn $line<'a>(
                    segment_ids: &impl Index<&'a str, Output = usize>,
                ) -> Line {
                    Line {
                        name: $name.to_string(),
                        color: Kind::$kind.color(),
                        kind: Kind::$kind,
                        path: paths::$line::$route(segment_ids),
                        trains: vec![
                            lines!(@trains $line, $route, $times),
                        ],
                    }
                }
            )*
        };
    }

    lines! {
        s3:                 "S3",           SuburbanRailway,
            hackescher_markt_bellevue, [7:24:54];
        u6:                 "U6",           UrbanRailway,
            naturkundemuseum_franzoesische_str, [5:55:40];
        tram_12:            "12",           Tram,
            oranienburger_tor_am_kupfergraben, [9:01:40];
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use na::Point2;

    use super::*;
    use crate::fixtures::{lines, paths};
    use crate::path::{NodeKind, SegmentRef};
    use common::{time, Order};

    #[test]
    fn test_getters() {
        let (_, segment_ids) = paths::tram_12::segments();
        let line = lines::tram_12(&segment_ids);
        assert_eq!(line.name(), "12");
        assert_eq!(line.kind(), Kind::Tram);
        let expected = &paths::tram_12::oranienburger_tor_am_kupfergraben(&segment_ids);
        assert_eq!(line.path(), expected);
    }

    #[test]
    fn test_fill_color_buffer() {
        let (_, segment_ids) = paths::tram_12::segments();
        let line = lines::tram_12(&segment_ids);
        let mut colors = Vec::new();
        line.fill_color_buffer(&mut colors);
        assert_relative_eq!(*colors, [0.8, 0.04, 0.13], epsilon = 0.01);
    }

    #[test]
    #[ignore]
    fn test_active_trains() {
        let (segments, segment_ids) = paths::tram_12::segments();
        let mut line = lines::tram_12(&segment_ids);
        assert_eq!(line.active_trains().count(), 0);
        line.update(&segments, time!(8:33:40));
        assert_eq!(line.active_trains().count(), 0);
        line.update(&segments, time!(0:01:00));
        assert_eq!(line.active_trains().count(), 1);
        line.update(&segments, time!(0:06:00));
        assert_eq!(line.active_trains().count(), 0);
        line.update(&segments, time!(0:22:00));
        assert_eq!(line.active_trains().count(), 1);
    }

    macro_rules! test_vertices {
        ($nodes:tt, $vertices:tt) => (
            test_vertices!($nodes, Railway, $vertices)
        );
        (
            [$($x:literal, $y:literal);* $(;)?],
            $kind:ident,
            [$($vertex:literal),* $(,)?]
        ) => (
            let segments = vec![
                Segment::new(vec![ $(
                    Node::new(Point2::new($x, $y), NodeKind::Waypoint)
                ),* ]),
            ];
            let path = SegmentedPath::new(vec![
                SegmentRef::new(0, Order::Forward),
            ]);
            let line = Line {
                name: String::new(),
                color: Kind::$kind.color(),
                kind: Kind::$kind,
                path,
                trains: Vec::new(),
            };
            let mut vertices = Vec::new();
            let mut lengths = Vec::new();
            line.fill_vertices_buffer_with_lengths(&segments, &mut vertices, &mut lengths);
            assert_relative_eq!(*vertices, [ $( $vertex ),* ]);
            assert_eq!(lengths, [vertices.len() / 2]);
        );
    }

    #[test]
    fn test_empty_vertices() {
        test_vertices!([], []);
    }

    #[test]
    fn test_straight_vertices() {
        test_vertices!([
               0.0,    0.0;
             100.0,    0.0;
             200.0,    0.0;
        ], [
               0.0,   25.0,    0.0,  -25.0,
             100.0,   25.0,  100.0,  -25.0,
             200.0,   25.0,  200.0,  -25.0,
        ]);
    }

    #[test]
    fn test_different_line_size() {
        test_vertices!([
               0.0,    0.0;
             100.0,    0.0;
             200.0,    0.0;
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
               0.0,    0.0;
             100.0,    0.0;
             100.0,  100.0;
        ], [
               0.0,   25.0,    0.0,  -25.0,
              75.0,   25.0,  125.0,  -25.0,
              75.0,  100.0,  125.0,  100.0,
        ]);
    }
}
