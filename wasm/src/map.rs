use std::convert::From;
use std::iter;

use na::Point2;
use wasm_bindgen::prelude::*;

use simulation::{Line, Station};

use crate::view::View;

#[wasm_bindgen]
pub struct Map {
    stations: Vec<Station>,
    lines: Vec<Line>,
}

impl Map {
    fn new(stations: Vec<Station>, lines: Vec<Line>) -> Self {
        Self { stations, lines }
    }
}

#[wasm_bindgen]
impl Map {
    pub fn parse(data: &[u8]) -> Map {
        let dataset = bincode::deserialize::<storage::Dataset>(data).unwrap();
        dataset.into()
    }

    pub fn update(&mut self, time_passed: u32) {
        for line in &mut self.lines {
            line.update(time_passed);
        }
    }

    pub fn find_station(&self, view: &View, x: f32, y: f32) -> Option<String> {
        let position = view.unproject(Point2::new(x, y));
        self.stations
            .iter()
            .find(|station| station.contains(position))
            .map(|station| station.name().to_string())
    }

    pub fn station_size(&self) -> usize {
        self.stations.len()
    }

    pub fn station_positions(&self) -> Vec<f32> {
        let mut buffer = Vec::new();
        for station in &self.stations {
            station.fill_vertice_buffer(&mut buffer);
        }
        buffer
    }

    pub fn line_colors(&self) -> Vec<f32> {
        let mut colors = Vec::new();
        for line in &self.lines {
            line.fill_color_buffer(&mut colors);
        }
        colors
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn line_vertices_with_sizes(&self) -> (Vec<f32>, Vec<usize>) {
        let mut vertices = Vec::new();
        let mut sizes = Vec::new();
        for line in &self.lines {
            line.fill_vertices_buffer_with_lengths(&mut vertices, &mut sizes);
        }
        (vertices, sizes)
    }

    pub fn track_run_sizes(&self) -> Vec<usize> {
        self.line_vertices_with_sizes().1
    }

    pub fn line_vertices(&self) -> Vec<f32> {
        self.line_vertices_with_sizes().0
    }

    pub fn line_names(&self) -> String {
        self.lines
            .iter()
            .map(Line::name)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn train_size(&self) -> usize {
        self.lines
            .iter()
            .map(|line| line.active_trains().count())
            .sum()
    }

    pub fn train_vertices(&self) -> Vec<f32> {
        let mut buffer = Vec::new();
        for line in &self.lines {
            for train in line.active_trains() {
                train.fill_vertice_buffer(&mut buffer, line.nodes());
            }
        }
        buffer
    }

    pub fn train_colors(&self) -> Vec<f32> {
        let mut colors = Vec::new();
        for line in &self.lines {
            for _ in 0..6 * line.active_trains().count() {
                line.fill_color_buffer(&mut colors);
            }
        }
        colors
    }

    pub fn train_line_numbers(&self) -> Vec<u16> {
        let mut buffer = Vec::new();
        for (line_number, line) in self.lines.iter().enumerate() {
            buffer.extend(iter::repeat(line_number as u16).take(6 * line.active_trains().count()));
        }
        buffer
    }

    pub fn train_sides(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        for line in &self.lines {
            for _ in 0..line.active_trains().count() {
                buffer.extend_from_slice(&[0, 1, 0, 0, 1, 1, 1, 0, 1, 1, 0, 0]);
            }
        }
        buffer
    }

    pub fn train_extents(&self) -> Vec<f32> {
        let mut buffer = Vec::new();
        for line in &self.lines {
            let extent = line.kind().train_size().data;

            for _ in 0..6 * line.active_trains().count() {
                buffer.extend_from_slice(&extent);
            }
        }
        buffer
    }
}

impl From<storage::Dataset> for Map {
    fn from(dataset: storage::Dataset) -> Map {
        let stations = dataset
            .stations
            .into_iter()
            .map(storage::Station::load)
            .collect::<Vec<_>>();

        let lines = dataset.lines.into_iter().map(storage::Line::load).collect();

        Map::new(stations, lines)
    }
}
