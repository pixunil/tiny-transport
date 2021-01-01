use std::iter;
use std::rc::Rc;

use crate::line::Line;
use crate::path::Segment;
use crate::station::Station;

#[derive(Debug, PartialEq)]
pub struct Dataset {
    stations: Vec<Rc<Station>>,
    segments: Vec<Segment>,
    lines: Vec<Line>,
}

impl Dataset {
    pub fn new(stations: Vec<Rc<Station>>, segments: Vec<Segment>, lines: Vec<Line>) -> Self {
        Self {
            stations,
            segments,
            lines,
        }
    }

    pub fn update(&mut self, time_passed: u32) {
        for line in &mut self.lines {
            line.update(&self.segments, time_passed);
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

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    fn segment_vertices_with_sizes(&self) -> (Vec<f32>, Vec<usize>) {
        let mut vertices = Vec::new();
        let mut sizes = Vec::new();
        for segment in &self.segments {
            segment.fill_vertices_buffer_with_lengths(&mut vertices, &mut sizes);
        }
        (vertices, sizes)
    }

    pub fn segment_sizes(&self) -> Vec<usize> {
        self.segment_vertices_with_sizes().1
    }

    pub fn segment_vertices(&self) -> Vec<f32> {
        self.segment_vertices_with_sizes().0
    }

    pub fn line_colors(&self) -> Vec<f32> {
        let mut colors = Vec::new();
        for line in &self.lines {
            line.fill_color_buffer(&mut colors);
        }
        colors
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
            let nodes = line.path().nodes(&self.segments);
            for train in line.active_trains() {
                train.fill_vertice_buffer(&mut buffer, &nodes);
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
    use common::fixtures_with_ids;

    macro_rules! datasets {
        ( $( $dataset:ident: {
                stations: [ $($station:ident),* $(,)? ],
                segments: [ $($segment:ident),* $(,)? ],
                lines: [ $($line:ident),* $(,)? ],
            } ),* $(,)? ) => {
            $(
                pub fn $dataset() -> Dataset {
                    let (segments, segment_ids) = fixtures_with_ids!(segments::{$($segment),*});
                    Dataset {
                        stations: vec![ $(Rc::new(stations::$station())),* ],
                        segments,
                        lines: vec![ $(lines::$line(&segment_ids)),* ],
                    }
                }
            )*
        }
    }

    datasets! {
        tram_12: {
            stations: [
                oranienburger_tor, friedrichstr, universitaetsstr, am_kupfergraben,
                georgenstr_am_kupfergraben,
            ],
            segments: [
                oranienburger_tor_friedrichstr, universitaetsstr_am_kupfergraben,
            ],
            lines: [tram_12],
        },
        hauptbahnhof_friedrichstr: {
            stations: [
                hauptbahnhof, friedrichstr, hackescher_markt, bellevue,
                naturkundemuseum, franzoesische_str, oranienburger_tor,
                universitaetsstr, am_kupfergraben,
            ],
            segments: [
                hackescher_markt_bellevue, naturkundemuseum_franzoesische_str,
                oranienburger_tor_friedrichstr, universitaetsstr_am_kupfergraben,
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
        assert_eq!(dataset.segment_count(), 2);
        assert_eq!(dataset.segment_sizes(), [12, 6]);
        assert_eq!(dataset.segment_vertices().len(), 2 * 18);
        assert_eq!(dataset.line_colors().len(), 3);
        assert_eq!(dataset.line_names(), "12".to_string());
    }
}
