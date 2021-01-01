use itertools::Itertools;
use na::Vector2;

use crate::path::Node;

#[derive(Debug, PartialEq)]
pub struct Segment {
    nodes: Vec<Node>,
}

impl Segment {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self { nodes }
    }

    pub(super) fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn fill_vertices_buffer_with_lengths(
        &self,
        vertices: &mut Vec<f32>,
        lengths: &mut Vec<usize>,
    ) {
        let length = vertices.len();
        self.fill_vertices_buffer(vertices);
        lengths.push((vertices.len() - length) / 2);
    }

    fn fill_vertices_buffer(&self, vertices: &mut Vec<f32>) {
        let mut segments = self
            .nodes
            .iter()
            .tuple_windows()
            .map(|(before, after)| after.position() - before.position())
            .collect::<Vec<_>>();
        if segments.is_empty() {
            return;
        }
        segments.insert(0, *segments.first().unwrap());
        segments.insert(segments.len(), *segments.last().unwrap());

        for (node, adjacent) in self.nodes.iter().zip_eq(segments.windows(2)) {
            let perp = adjacent[0].perp(&adjacent[1]);
            let miter = if perp == 0.0 {
                Vector2::new(-adjacent[0].y, adjacent[0].x).normalize()
            } else {
                let preceding = adjacent[0] * adjacent[1].norm();
                let following = adjacent[1] * adjacent[0].norm();
                (following - preceding) / perp
            };

            Self::add_node_vertices_to_buffer(node, miter, vertices);
        }
    }

    fn add_node_vertices_to_buffer(node: &Node, mut miter: Vector2<f32>, vertices: &mut Vec<f32>) {
        miter *= 25.0;
        vertices.extend((node.position() + miter).iter());
        vertices.extend((node.position() - miter).iter());
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use std::rc::Rc;

    use na::Point2;

    use super::*;
    use crate::fixtures::stations;
    use crate::path::node::Kind;

    macro_rules! segments {
        (@kind $station:ident) => (
            Kind::Stop { at: Rc::new(stations::$station()) }
        );
        (@kind) => ( Kind::Waypoint );
        ($( $segment:ident : [ $( $x:literal, $y:literal $( , $station:ident )? );* $(;)? ] ),* $(,)?) => {
            $(
                pub fn $segment<'a>() -> Segment {
                    Segment {
                        nodes: vec![ $(
                            Node::new(
                                Point2::new($x as f32, $y as f32),
                                segments!(@kind $($station)?),
                            )
                        ),* ],
                    }
                }
            )*
        }
    }

    segments! {
        hackescher_markt_bellevue: [
               846,  -1428, hackescher_markt;
              -244,  -1229, friedrichstr;
             -1387,  -1700, hauptbahnhof;
             -2893,  -1178, bellevue;
        ],
        naturkundemuseum_franzoesische_str: [
              -491,  -2348, naturkundemuseum;
              -164,  -1784, oranienburger_tor;
              -111,  -1115, friedrichstr;
               -55,   -558, franzoesische_str;
        ],
        zingster_str: [
              7204,  -6855;
              7269,  -6742, zingster_str;
        ],
        zingster_str_ribnitzer_str: [
              7335,  -6629;
              7337,  -6741;
        ],
        zingster_str_ribnitzer_str_prerower_platz: [
              7400,  -6517, zingster_str_ribnitzer_str;
              7662,  -6066, ahrenshooper_str;
              7795,  -5952;
              7926,  -5727, prerower_platz;
        ],
        oranienburger_tor_friedrichstr: [
               -98,  -1671, oranienburger_tor;
              -101,  -1560;
              -108,  -1226;
              -111,  -1115, friedrichstr;
               -46,  -1003;
                22,  -1001;
        ],
        universitaetsstr_am_kupfergraben: [
                90,   -999;
               158,   -998, universitaetsstr;
               429,   -992, am_kupfergraben;
        ],
        am_kupfergraben_georgenstr: [
               429,   -992, am_kupfergraben;
               432,  -1103;
               367,  -1216;
               299,  -1217;
               228,  -1108, georgenstr_am_kupfergraben;
                93,  -1111;
                25,  -1112;
        ],
        weskammstr_waldsassener_str: [
             -2958,  10616, weskammstr;
             -2958,  10616;
             -2961,  10727;
             -2963,  10838, lichterfelder_ring_waldsassener_str;
             -2966,  10949;
             -2968,  11061;
             -2903,  11173;
             -2906,  11285, waldsassener_str;
             -2906,  11285;
             -2906,  11285;
        ],
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;
    use na::Point2;

    use super::*;
    use crate::path::NodeKind;

    macro_rules! test_vertices {
        ($nodes:tt, $vertices:tt) => (
            test_vertices!($nodes, Railway, $vertices)
        );
        (
            [$($x:literal, $y:literal);* $(;)?],
            $kind:ident,
            [$($vertex:literal),* $(,)?]
        ) => (
            let segment = Segment::new(vec![ $(
                Node::new(Point2::new($x, $y), NodeKind::Waypoint)
            ),* ]);
            let mut vertices = Vec::new();
            let mut lengths = Vec::new();
            segment.fill_vertices_buffer_with_lengths(&mut vertices, &mut lengths);
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
    #[ignore]
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
