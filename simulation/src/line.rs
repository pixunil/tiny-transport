use na::{Point2, Vector2};

use crate::color::Color;
use crate::train::Train;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
enum LineNodeKind {
    Waypoint,
    Stop,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LineNode {
    position: Point2<f32>,
    kind: LineNodeKind,
}

impl LineNode {
    pub fn new(position: Point2<f32>) -> LineNode {
        LineNode {
            position,
            kind: LineNodeKind::Waypoint,
        }
    }

    pub fn promote_to_stop(&mut self) {
        self.kind = LineNodeKind::Stop;
    }

    pub fn is_stop(&self) -> bool {
        self.kind == LineNodeKind::Stop
    }

    pub fn position(&self) -> Point2<f32> {
        self.position
    }
}

#[derive(Debug)]
pub struct Line {
    name: String,
    nodes: Vec<LineNode>,
    trains: Vec<Train>,
}

impl Line {
    pub fn new(name: String, nodes: Vec<LineNode>, trains: Vec<Train>) -> Line {
        Line {
            name,
            nodes,
            trains,
        }
    }

    pub fn update(&mut self, time: u32) {
        for train in &mut self.trains {
            train.update(time);
        }
    }

    fn train_size(&self) -> usize {
        self.trains.iter()
            .filter(|train| train.is_active())
            .count()
    }
}

#[derive(Debug)]
pub struct LineGroup {
    color: Color,
    lines: Vec<Line>,
}

impl LineGroup {
    pub fn new(color: Color, lines: Vec<Line>) -> LineGroup {
        LineGroup { color, lines }
    }

    pub fn update(&mut self, time: u32) {
        for line in &mut self.lines {
            line.update(time);
        }
    }

    pub fn color_buffer_data(&self) -> impl Iterator<Item=f32> + '_ {
        self.color.iter().map(|component| component as f32 / 255.0)
    }

    pub fn track_runs_size(&self) -> usize {
        self.lines.len()
    }

    pub fn fill_vertice_buffer_sizes(&self, buffer: &mut Vec<usize>) {
        for line in &self.lines {
            buffer.push(2 * line.nodes.len());
        }
    }

    pub fn fill_vertice_buffer_data(&self, buffer: &mut Vec<f32>) {
        for line in &self.lines {
            let mut segments = line.nodes.windows(2)
                .map(|segment| segment[1].position - segment[0].position)
                .collect::<Vec<_>>();
            segments.insert(0, segments.first().unwrap().clone());
            segments.insert(segments.len(), segments.last().unwrap().clone());

            for (waypoint, adjacent) in line.nodes.iter().zip(segments.windows(2)) {
                let perp = adjacent[0].perp(&adjacent[1]);
                let miter = if perp == 0.0 {
                    Vector2::new(-adjacent[0].y, adjacent[0].x).normalize()
                } else {
                    let preceding = adjacent[0] * adjacent[1].norm();
                    let following = adjacent[1] * adjacent[0].norm();
                    (following - preceding) / perp
                };

                buffer.extend((waypoint.position + miter).iter());
                buffer.extend((waypoint.position - miter).iter());
            }
        }
    }

    pub fn train_size(&self) -> usize {
        self.lines.iter()
            .map(Line::train_size)
            .sum()
    }

    pub fn fill_train_vertice_buffer(&self, buffer: &mut Vec<f32>) {
        for line in &self.lines {
            for train in &line.trains {
                if train.is_active() {
                    train.fill_vertice_buffer(buffer, &line.nodes);
                }
            }
        }
    }

    pub fn fill_train_color_buffer(&self, buffer: &mut Vec<f32>) {
        for _ in 0..6 * self.train_size() {
            buffer.extend(self.color_buffer_data());
        }
    }
}
