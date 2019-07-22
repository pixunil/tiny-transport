use std::rc::Rc;
use std::ops::Deref;
use std::collections::{HashSet, HashMap};

use crate::color::Color;
use crate::station::Station;
use crate::track::{Connection, Track, TrackBundle};
use crate::train::Train;

#[derive(Debug)]
pub struct Stop {
    station: Rc<Station>,
    terminus: bool,
}

impl Stop {
    fn new(station: Rc<Station>) -> Stop {
        Stop {
            station,
            terminus: false,
        }
    }

    fn make_terminus(&mut self) {
        self.terminus = true;
    }

    pub fn is_terminus(&self) -> bool {
        self.terminus
    }
}

impl Deref for Stop {
    type Target = Station;

    fn deref(&self) -> &Station {
        &self.station
    }
}

#[derive(Debug)]
pub struct Line {
    name: String,
    stops: Vec<Rc<Stop>>,
    tracks: Vec<Track>,
    trains: Vec<Train>,
}

impl Line {
    pub fn new(name: String, color: Color, stations: Vec<Rc<Station>>, trains: Vec<Train>) -> Line {
        let mut stops = stations.into_iter()
            .map(|station| Stop::new(station))
            .collect::<Vec<_>>();
        stops.first_mut().map(Stop::make_terminus);
        stops.last_mut().map(Stop::make_terminus);
        let stops = stops.into_iter()
            .map(|stop| Rc::new(stop))
            .collect::<Vec<_>>();
        let tracks = stops.windows(2)
            .map(|connection| Track::new(connection[0].clone(), connection[1].clone(), color.clone()))
            .collect();
        Line {
            name,
            stops,
            tracks,
            trains,
        }
    }

    pub fn update(&mut self, time: u32) {
        for train in &mut self.trains {
            train.update(time);
        }
    }

    pub fn attach_tracks(&self, track_bundles: &mut HashMap<Connection, TrackBundle>) {
        for track in &self.tracks {
            track.attach_to(track_bundles);
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

    pub fn attach_tracks(&self, track_bundles: &mut HashMap<Connection, TrackBundle>) {
        for line in &self.lines {
            line.attach_tracks(track_bundles);
        }
    }

    pub fn update(&mut self, time: u32) {
        for line in &mut self.lines {
            line.update(time);
        }
    }

    pub fn color_buffer_data(&self) -> impl Iterator<Item=f32> + '_ {
        self.color.iter().map(|component| component as f32 / 255.0)
    }

    fn build_track_runs(&self) -> Vec<Vec<Track>> {
        let mut track_runs = Vec::new();
        let mut tracks = HashSet::new();
        for line in &self.lines {
            let mut track_run = Vec::new();
            for track in &line.tracks {
                if !tracks.contains(&track.key()) {
                    tracks.insert(track.key());
                    track_run.push(track.clone());
                } else if !track_run.is_empty() {
                    track_runs.push(track_run);
                    track_run = Vec::new();
                }
            }

            if !track_run.is_empty() {
                track_runs.push(track_run);
            }
        }

        track_runs
    }

    pub fn track_runs_size(&self) -> usize {
        self.build_track_runs().len()
    }

    pub fn fill_vertice_buffer_sizes(&self, buffer: &mut Vec<usize>) {
        for track_run in self.build_track_runs() {
            buffer.push(4 * track_run.len());
        }
    }

    pub fn fill_vertice_buffer_data(&self, buffer: &mut Vec<f32>, track_bundles: &HashMap<Connection, TrackBundle>) {
        for track_run in self.build_track_runs() {
            for track in track_run {
                track.fill_vertice_buffer_data(buffer, track_bundles);
            }
        }
    }

    pub fn train_size(&self) -> usize {
        self.lines.iter()
            .map(Line::train_size)
            .sum()
    }

    pub fn fill_train_vertice_buffer(&self, buffer: &mut Vec<f32>, track_bundles: &HashMap<Connection, TrackBundle>) {
        for line in &self.lines {
            for train in &line.trains {
                if train.is_active() {
                    train.fill_vertice_buffer(buffer, &line.tracks, track_bundles);
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
