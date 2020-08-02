use std::iter;
use std::rc::Rc;

use crate::line::Line;
use crate::station::Station;

#[derive(Debug, PartialEq)]
pub struct Dataset {
    stations: Vec<Rc<Station>>,
    lines: Vec<Line>,
}

impl Dataset {
    pub fn new(stations: Vec<Rc<Station>>, lines: Vec<Line>) -> Self {
        Self { stations, lines }
    }

    pub fn update(&mut self, time_passed: u32) {
        for line in &mut self.lines {
            line.update(time_passed);
        }
    }

    pub fn station(&self, index: usize) -> &Station {
        self.stations[index].as_ref()
    }

    pub fn station_count(&self) -> usize {
        self.stations.len()
    }

    pub fn station_positions(&self) -> Vec<f32> {
        let mut buffer = Vec::new();
        for station in &self.stations {
            station.fill_vertice_buffer(&mut buffer);
        }
        buffer
    }

    pub fn station_types(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        for station in &self.stations {
            station.fill_type_buffer(&mut buffer);
        }
        buffer
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line_colors(&self) -> Vec<f32> {
        let mut colors = Vec::new();
        for line in &self.lines {
            line.fill_color_buffer(&mut colors);
        }
        colors
    }

    fn line_vertices_with_sizes(&self) -> (Vec<f32>, Vec<usize>) {
        let mut vertices = Vec::new();
        let mut sizes = Vec::new();
        for line in &self.lines {
            line.fill_vertices_buffer_with_lengths(&mut vertices, &mut sizes);
        }
        (vertices, sizes)
    }

    pub fn line_vertices_sizes(&self) -> Vec<usize> {
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

    pub fn train_count(&self) -> usize {
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

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use super::*;
    use crate::fixtures::{lines, stations};

    macro_rules! datasets {
        ( $( $dataset:ident => {
                stations: [ $($station:ident),* $(,)? ],
                lines: [ $($line:ident),* $(,)? ],
            } ),* $(,)? ) => {
            $(
                pub fn $dataset() -> Dataset {
                    Dataset {
                        stations: vec![ $(Rc::new(stations::$station())),* ],
                        lines: vec![ $(lines::$line()),* ],
                    }
                }
            )*
        }
    }

    datasets! {
        tram_12 => {
            stations: [
                oranienburger_tor, friedrichstr, universitaetsstr, am_kupfergraben,
                georgenstr_am_kupfergraben,
            ],
            lines: [tram_12],
        },
        hauptbahnhof_friedrichstr => {
            stations: [
                hauptbahnhof, friedrichstr, hackescher_markt, bellevue,
                naturkundemuseum, franzoesische_str, oranienburger_tor,
                universitaetsstr, am_kupfergraben, georgenstr_am_kupfergraben,
            ],
            lines: [u6, s3, tram_12],
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::fixtures::{datasets, stations};

    #[test]
    fn test_static_data() {
        let dataset = datasets::tram_12();
        assert_eq!(dataset.station(0), &stations::oranienburger_tor());
        assert_eq!(dataset.station_count(), 5);
        assert_eq!(dataset.station_positions().len(), 2 * 5);
        assert_eq!(dataset.station_types().len(), 5);
        assert_eq!(dataset.line_count(), 1);
        assert_eq!(dataset.line_colors().len(), 3);
        assert_eq!(dataset.line_vertices_sizes(), [20, 28]);
        assert_eq!(dataset.line_vertices().len(), 2 * 48);
        assert_eq!(dataset.line_names(), "12".to_string());
    }
}
