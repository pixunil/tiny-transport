use std::rc::Rc;
use std::collections::HashMap;
use std::cmp::{min, max};

use na::{Point2, Vector2};

use crate::color::Color;
use crate::line::Stop;

#[derive(Debug)]
pub struct TrackBundle {
    orientation: Vector2<f32>,
    tracks: HashMap<Color, u8>,
}

impl TrackBundle {
    pub fn new(orientation: Vector2<f32>) -> TrackBundle {
        TrackBundle {
            orientation,
            tracks: HashMap::new(),
        }
    }

    pub fn attach(&mut self, color: Color) {
        let number = self.tracks.len() as u8;
        self.tracks.entry(color).or_insert(number);
    }

    fn vertical(&self, color: &Color) -> f32 {
        self.tracks[color] as f32 - (self.tracks.len() - 1) as f32 / 2.0
    }
}

pub type Connection = (usize, usize);

#[derive(Clone, Debug)]
pub struct Track {
    from: Rc<Stop>,
    to: Rc<Stop>,
    color: Color,
}

impl Track {
    pub fn new(from: Rc<Stop>, to: Rc<Stop>, color: Color) -> Track {
        Track {
            from,
            to,
            color,
        }
    }

    pub fn direction(&self) -> Vector2<f32> {
        self.to.position() - self.from.position()
    }

    fn offset(&self) -> Vector2<f32> {
        self.direction().normalize() * 2.0
    }

    pub fn orthogonal(&self) -> Vector2<f32> {
        let direction = self.direction();
        Vector2::new(-direction.y, direction.x)
    }

    pub fn key(&self) -> (usize, usize) {
        (min(self.from.id(), self.to.id()), max(self.from.id(), self.to.id()))
    }

    pub fn attach_to(&self, track_bundles: &mut HashMap<Connection, TrackBundle>) {
        track_bundles.entry(self.key())
            .or_insert_with(|| TrackBundle::new(self.orthogonal()))
            .attach(self.color.clone());
    }

    pub fn fill_vertice_buffer_data(&self, buffer: &mut Vec<f32>, track_bundles: &HashMap<Connection, TrackBundle>) {
        let track_bundle = &track_bundles[&self.key()];
        self.fill_stop_vertices(buffer, track_bundle, &self.from, 1.0);
        self.fill_stop_vertices(buffer, track_bundle, &self.to, -1.0);
    }

    fn fill_stop_vertices(&self, buffer: &mut Vec<f32>, track_bundle: &TrackBundle, stop: &Stop, offset: f32) {
        let position = stop.position();
        let normal = track_bundle.orientation.normalize();
        let direction = if stop.is_terminus() {
            Vector2::zeros()
        } else {
            offset * self.offset()
        };
        let vertical = track_bundle.vertical(&self.color);

        buffer.extend((position + direction + normal * (vertical - 0.5)).iter());
        buffer.extend((position + direction + normal * (vertical + 0.5)).iter());
    }

    pub fn interpolated_position(&self, preceding: Option<&Track>, following: Option<&Track>, track_bundles: &HashMap<Connection, TrackBundle>, interpolation: f32) -> (Point2<f32>, Vector2<f32>) {
        let track_bundle = &track_bundles[&self.key()];
        let normal = track_bundle.orientation.normalize();
        let vertical = track_bundle.vertical(&self.color);
        let offset = self.offset() + normal * vertical;

        let preceding_direction = preceding.map_or_else(|| self.offset(), |preceding| {
            Track::edge_direction(preceding, self, track_bundles)
        });
        let following_direction = following.map_or_else(|| self.offset(), |following| {
            Track::edge_direction(self, following, track_bundles)
        });

        let direction = self.direction() - 2.0 * self.offset();

        let length = direction.norm() + preceding_direction.norm() + following_direction.norm();
        let distance = interpolation * length;
        let vector = if distance < preceding_direction.norm() {
            (distance / preceding_direction.norm() - 1.0) * preceding_direction
        } else if distance < preceding_direction.norm() + direction.norm() {
            (distance - preceding_direction.norm()) / direction.norm() * direction
        } else {
            direction + (distance - preceding_direction.norm() - direction.norm()) / following_direction.norm() * following_direction
        };

        let orientation = if distance < preceding_direction.norm() {
            preceding_direction
        } else if distance < preceding_direction.norm() + direction.norm() {
            direction
        } else {
            following_direction
        };

        (self.from.position() + offset + vector, orientation.normalize())
    }

    fn edge_direction(incoming: &Track, outgoing: &Track, track_bundles: &HashMap<Connection, TrackBundle>) -> Vector2<f32> {
        let track_bundle = &track_bundles[&incoming.key()];
        let normal = track_bundle.orientation.normalize();
        let vertical = track_bundle.vertical(&incoming.color);
        let incoming = incoming.offset() - normal * vertical;

        let track_bundle = &track_bundles[&outgoing.key()];
        let normal = track_bundle.orientation.normalize();
        let vertical = track_bundle.vertical(&outgoing.color);
        let outgoing = outgoing.offset() + normal * vertical;

        (incoming + outgoing) / 2.0
    }
}
