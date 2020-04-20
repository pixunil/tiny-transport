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

    pub fn lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn active_train_count(&self) -> usize {
        self.lines.iter()
            .map(|line| line.active_trains().count())
            .sum()
    }
}
