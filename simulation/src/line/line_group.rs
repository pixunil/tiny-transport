use crate::color::Color;
use super::Line;

#[derive(Debug)]
pub struct LineGroup {
    color: Color,
    lines: Vec<Line>,
}

impl LineGroup {
    pub fn new(color: Color, lines: Vec<Line>) -> LineGroup {
        LineGroup { color, lines }
    }

    pub fn update(&mut self, time_passed: u32) {
        for line in &mut self.lines {
            line.update(time_passed);
        }
    }

    pub fn color_buffer_data(&self) -> impl Iterator<Item=f32> + '_ {
        self.color.iter().map(|component| component as f32 / 255.0)
    }

    pub fn track_runs_size(&self) -> usize {
        2 * self.lines.len()
    }

    pub fn fill_vertices_buffer_with_lengths(&self, vertices: &mut Vec<f32>, lengths: &mut Vec<usize>) {
        for line in &self.lines {
            line.fill_vertices_buffer_with_lengths(vertices, lengths);
        }
    }

    pub fn train_size(&self) -> usize {
        self.lines.iter()
            .map(|line| line.active_trains().count())
            .sum()
    }

    pub fn fill_train_vertice_buffer(&self, buffer: &mut Vec<f32>) {
        for line in &self.lines {
            for train in line.active_trains() {
                train.fill_vertice_buffer(buffer, &line.nodes());
            }
        }
    }

    pub fn fill_train_color_buffer(&self, buffer: &mut Vec<f32>) {
        for _ in 0..6 * self.train_size() {
            buffer.extend(self.color_buffer_data());
        }
    }
}
