use std::convert::From;

use na::Point2;
use wasm_bindgen::prelude::*;

use simulation::{Station, LineGroup};
use serialization::Dataset;

use crate::view::View;

#[wasm_bindgen]
pub struct Map {
    stations: Vec<Station>,
    line_groups: Vec<LineGroup>,
}

impl Map {
    fn new(stations: Vec<Station>, line_groups: Vec<LineGroup>) -> Map {
        Map {
            stations,
            line_groups,
        }
    }
}

#[wasm_bindgen]
impl Map {
    pub fn parse(data: &[u8]) -> Map {
        let dataset = bincode::deserialize::<Dataset>(data).unwrap();
        dataset.into()
    }

    pub fn update(&mut self, time_passed: u32) {
        for line_group in &mut self.line_groups {
            line_group.update(time_passed);
        }
    }

    pub fn find_station(&self, view: &View, x: f32, y: f32) -> Option<String> {
        let position = view.unproject(Point2::new(x, y));
        self.stations.iter()
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
        self.line_groups.iter()
            .flat_map(|line_group| line_group.color_buffer_data())
            .collect()
    }

    pub fn line_sizes(&self) -> Vec<usize> {
        self.line_groups.iter()
            .map(|line_group| line_group.track_runs_size())
            .collect()
    }

    fn line_vertices_with_sizes(&self) -> (Vec<f32>, Vec<usize>) {
        let mut vertices = Vec::new();
        let mut sizes = Vec::new();
        for line_group in &self.line_groups {
            line_group.fill_vertices_buffer_with_lengths(&mut vertices, &mut sizes);
        }
        (vertices, sizes)
    }

    pub fn track_run_sizes(&self) -> Vec<usize> {
        self.line_vertices_with_sizes().1
    }

    pub fn line_vertices(&self) -> Vec<f32> {
        self.line_vertices_with_sizes().0
    }

    pub fn train_size(&self) -> usize {
        self.line_groups.iter()
            .map(LineGroup::train_size)
            .sum()
    }

    pub fn train_vertices(&self) -> Vec<f32> {
        let mut buffer = Vec::new();
        for line_group in &self.line_groups {
            line_group.fill_train_vertice_buffer(&mut buffer);
        }
        buffer
    }

    pub fn train_colors(&self) -> Vec<f32> {
        let mut buffer = Vec::new();
        for line_group in &self.line_groups {
            line_group.fill_train_color_buffer(&mut buffer);
        }
        buffer
    }
}

impl From<Dataset> for Map {
    fn from(dataset: Dataset) -> Map {
        let stations = dataset.stations.into_iter()
            .map(serialization::Station::unfreeze)
            .collect::<Vec<_>>();

        let line_groups = dataset.line_groups.into_iter()
            .map(|line_group| line_group.unfreeze())
            .collect();

        Map::new(stations, line_groups)
    }
}
