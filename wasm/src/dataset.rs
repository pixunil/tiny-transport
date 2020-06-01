use na::Point2;
use wasm_bindgen::prelude::*;

use crate::view::View;

#[wasm_bindgen]
pub struct Dataset {
    inner: simulation::Dataset,
}

#[wasm_bindgen]
impl Dataset {
    pub fn parse(data: &[u8]) -> Self {
        let dataset = bincode::deserialize::<storage::Dataset>(data).unwrap();
        Self {
            inner: dataset.load(),
        }
    }

    pub fn update(&mut self, time_passed: u32) {
        self.inner.update(time_passed)
    }

    #[wasm_bindgen(js_name = findStation)]
    pub fn find_station(&self, view: &View, x: f32, y: f32) -> Option<String> {
        let position = view.unproject(Point2::new(x, y));
        self.inner
            .find_station(position)
            .map(|station| station.name().to_string())
    }
}

macro_rules! delegate {
    ($( pub fn $name:ident(&self) -> $return_type:ty; [$js_name:ident] )*) => {
        #[wasm_bindgen]
        impl Dataset {
            $(
                #[wasm_bindgen(js_name = $js_name)]
                pub fn $name(&self) -> $return_type {
                    self.inner.$name()
                }
            )*
        }
    }
}

delegate! {
    pub fn station_count(&self) -> usize; [stationCount]
        pub fn station_positions(&self) -> Vec<f32>; [stationPositions]

        pub fn line_count(&self) -> usize; [lineCount]
        pub fn line_colors(&self) -> Vec<f32>; [lineColors]
        pub fn line_vertices_sizes(&self) -> Vec<usize>; [lineVerticesSizes]
        pub fn line_vertices(&self) -> Vec<f32>; [lineVertices]
        pub fn line_names(&self) -> String; [lineNames]

        pub fn train_count(&self) -> usize; [trainCount]
        pub fn train_vertices(&self) -> Vec<f32>; [trainVertices]
        pub fn train_colors(&self) -> Vec<f32>; [trainColors]
        pub fn train_line_numbers(&self) -> Vec<u16>; [trainLineNumbers]
        pub fn train_sides(&self) -> Vec<u8>; [trainSides]
        pub fn train_extents(&self) -> Vec<f32>; [trainExtents]
}
