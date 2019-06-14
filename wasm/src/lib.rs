extern crate nalgebra as na;
extern crate gtfs_sim_simulation as simulation;

use std::rc::Rc;
use std::convert::From;
use std::collections::HashMap;

use na::Point2;
use wasm_bindgen::prelude::*;

use simulation::{Dataset, Station, LineGroup, Connection, TrackBundle};

#[wasm_bindgen]
pub struct Map {
    stations: Vec<Rc<Station>>,
    line_groups: Vec<LineGroup>,
    track_bundles: HashMap<Connection, TrackBundle>,
}

impl Map {
    fn new(stations: Vec<Rc<Station>>, line_groups: Vec<LineGroup>) -> Map {
        let mut track_bundles = HashMap::new();
        for line_group in &line_groups {
            line_group.attach_tracks(&mut track_bundles);
        }

        Map {
            stations,
            line_groups,
            track_bundles,
        }
    }
}

#[wasm_bindgen]
impl Map {
    pub fn parse(data: &[u8]) -> Map {
        let dataset = bincode::deserialize::<Dataset>(data).unwrap();
        dataset.into()
    }

    pub fn update(&mut self, time: u32) {
        for line_group in &mut self.line_groups {
            line_group.update(time);
        }
    }

    pub fn find_station(&self, x: f32, y: f32) -> Option<String> {
        let position = Point2::new(x, y);
        self.stations.iter()
            .find(|station| station.contains(&position))
            .map(|station| station.name().into())
    }

    pub fn station_size(&self) -> usize {
        self.stations.len()
    }

    pub fn station_positions(&self) -> Vec<f32> {
        self.stations.iter()
            .flat_map(|station| station.position_buffer_data())
            .collect()
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

    pub fn track_run_sizes(&self) -> Vec<usize> {
        let mut buffer = Vec::new();
        for line_group in &self.line_groups {
            line_group.fill_vertice_buffer_sizes(&mut buffer);
        }
        buffer
    }

    pub fn line_vertices(&self) -> Vec<f32> {
        let mut buffer = Vec::new();
        for line_group in &self.line_groups {
            line_group.fill_vertice_buffer_data(&mut buffer, &self.track_bundles);
        }
        buffer
    }

    pub fn train_size(&self) -> usize {
        self.line_groups.iter()
            .map(LineGroup::train_size)
            .sum()
    }

    pub fn train_vertices(&self) -> Vec<f32> {
        let mut buffer = Vec::new();
        for line_group in &self.line_groups {
            line_group.fill_train_vertice_buffer(&mut buffer, &self.track_bundles);
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
            .map(|station| Rc::new(station))
            .collect::<Vec<_>>();

        let line_groups = dataset.line_groups.into_iter()
            .map(|line_group| line_group.bind(&stations))
            .collect();

        Map::new(stations, line_groups)
    }
}

#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen(start)]
pub fn main() {
    ::std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}
