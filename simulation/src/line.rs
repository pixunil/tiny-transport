use std::rc::Rc;
use std::ops::Deref;
use std::collections::HashMap;

use na::{Point2, Vector2};

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
    shape: Vec<Point2<f32>>,
    tracks: Vec<Track>,
    trains: Vec<Train>,
}

impl Line {
    pub fn new(name: String, color: Color, stations: Vec<Rc<Station>>, shape: Vec<Point2<f32>>, trains: Vec<Train>) -> Line {
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
            shape,
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

    pub fn track_runs_size(&self) -> usize {
        self.lines.len()
    }

    pub fn fill_vertice_buffer_sizes(&self, buffer: &mut Vec<usize>) {
        for line in &self.lines {
            buffer.push(2 * line.shape.len());
        }
    }

    pub fn fill_vertice_buffer_data(&self, buffer: &mut Vec<f32>) {
        for line in &self.lines {
            let mut segments = line.shape.windows(2)
                .map(|segment| segment[1] - segment[0])
                .collect::<Vec<_>>();
            segments.insert(0, segments.first().unwrap().clone());
            segments.insert(segments.len(), segments.last().unwrap().clone());

            for (waypoint, adjacent) in line.shape.iter().zip(segments.windows(2)) {
                let perp = adjacent[0].perp(&adjacent[1]);
                let miter = if perp == 0.0 {
                    Vector2::new(-adjacent[0].y, adjacent[0].x).normalize()
                } else {
                    let preceding = adjacent[0] * adjacent[1].norm();
                    let following = adjacent[1] * adjacent[0].norm();
                    (following - preceding) / perp
                };

                buffer.extend((waypoint + miter).iter());
                buffer.extend((waypoint - miter).iter());
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
