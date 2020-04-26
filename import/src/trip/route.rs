use std::rc::Rc;

use chrono::NaiveDate;

use super::{Node, Trip};
use crate::location::Location;

#[derive(Debug, PartialEq)]
pub struct Route {
    nodes: Vec<Node>,
    trips: Vec<Trip>,
}

impl Route {
    pub(super) fn new(nodes: Vec<Node>, trips: Vec<Trip>) -> Route {
        Route { nodes, trips }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter()
    }

    pub(crate) fn num_trips_at(&self, date: NaiveDate) -> usize {
        self.trips
            .iter()
            .filter(|trip| trip.available_at(date))
            .count()
    }

    pub(crate) fn locations(&self) -> impl Iterator<Item = &Rc<Location>> {
        self.nodes.iter().filter_map(Node::location)
    }

    pub(crate) fn freeze_nodes(&self) -> Vec<simulation::Node> {
        self.nodes.iter().map(Node::freeze).collect()
    }

    pub(crate) fn freeze_trains(&self, date: NaiveDate) -> Vec<serialization::Train> {
        self.trips
            .iter()
            .filter(|trip| trip.available_at(date))
            .map(|trip| trip.freeze())
            .collect()
    }
}
