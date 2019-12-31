use std::rc::Rc;

use chrono::NaiveDate;

use na::Point2;

use simulation::LineNode;
use crate::shape::Shape;
use crate::location::Location;
use super::Trip;

#[derive(Debug, PartialEq)]
pub(crate) struct Route {
    pub(crate) locations: Vec<Rc<Location>>,
    shape: Shape,
    trips: Vec<Trip>,
}

impl Route {
    pub(super) fn new(locations: Vec<Rc<Location>>, shape: Shape) -> Route {
        Route {
            locations,
            shape,
            trips: Vec::new(),
        }
    }

    pub(super) fn add_trip(&mut self, trip: Trip) {
        self.trips.push(trip);
    }

    pub(crate) fn num_trips_at(&self, date: NaiveDate) -> usize {
        self.trips.iter()
            .filter(|trip| trip.available_at(date))
            .count()
    }

    pub(crate) fn freeze_nodes(&self) -> Vec<LineNode> {
        let mut nodes = self.shape.iter()
            .map(|waypoint| {
                let x = 2000.0 * (waypoint.x - 13.5);
                let y = -2000.0 * (waypoint.y - 105.04);
                LineNode::new(Point2::new(x, y))
            })
            .collect::<Vec<_>>();

        let mut lower = 0;
        for station in &self.locations[1 ..] {
            let (pos, node) = nodes.iter_mut()
                .enumerate()
                .skip(lower)
                .min_by(|(_, a), (_, b)| {
                    let a = na::distance(&station.position(), &a.position());
                    let b = na::distance(&station.position(), &b.position());
                    a.partial_cmp(&b).unwrap()
                })
                .unwrap();
            node.promote_to_stop();
            lower = pos;
        }

        let station = &self.locations[0];
        let second_stop = nodes.iter().position(LineNode::is_stop).unwrap();

        let node = nodes[.. second_stop].iter_mut()
            .min_by(|a, b| {
                let a = na::distance(&station.position(), &a.position());
                let b = na::distance(&station.position(), &b.position());
                a.partial_cmp(&b).unwrap()
            })
            .unwrap();
        node.promote_to_stop();

        nodes
    }

    pub(crate) fn freeze_trains(&self, date: NaiveDate) -> Vec<serialization::Train> {
        self.trips.iter()
            .filter(|trip| trip.available_at(date))
            .map(|trip| trip.freeze())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_abs_diff_eq;

    use crate::{station, shape};

    #[macro_export]
    macro_rules! route {
        ([$($location:ident),*], $shape:ident, [$(($trip:ident, $direction:ident, $start:expr)),*]) => ({
            let locations = vec![$(Rc::new($crate::station!($location))),*];
            #[allow(unused_mut)]
            let mut route = $crate::trip::Route::new(locations, $crate::shape!($shape));
            $(
                route.add_trip($crate::trip!($trip, $direction, $start));
            )*
            route
        });
        (blue, $trips:tt) => (
            $crate::route!([main_station, center, market], blue, $trips)
        );
    }

    #[test]
    fn test_freeze_nodes_exact_shape() {
        let shape = shape!(52.526, 13.369; 52.523, 13.378; 52.520, 13.387; 52.521, 13.394; 52.523, 13.402);
        let route = Route::new(station![main_station, center, market], shape);
        let mut expected_nodes = [
            LineNode::new(Point2::new(-262.0, -24.0)),
            LineNode::new(Point2::new(-244.0, -12.0)),
            LineNode::new(Point2::new(-226.0,   0.0)),
            LineNode::new(Point2::new(-212.0,  -4.0)),
            LineNode::new(Point2::new(-196.0, -12.0)),
        ];
        expected_nodes[0].promote_to_stop();
        expected_nodes[2].promote_to_stop();
        expected_nodes[4].promote_to_stop();
        assert_abs_diff_eq!(*route.freeze_nodes(), expected_nodes, epsilon = 0.01);
    }

    #[test]
    fn test_freeze_nodes_circle() {
        let shape = shape!(52.549, 13.388; 52.503, 13.469; 52.475, 13.366; 52.501, 13.283; 52.549, 13.388);
        let route = Route::new(station![north_cross, east_cross, south_cross, west_cross, north_cross], shape);
        let mut expected_nodes = [
            LineNode::new(Point2::new(-224.0, -116.0)),
            LineNode::new(Point2::new( -62.0,   68.0)),
            LineNode::new(Point2::new(-268.0,  180.0)),
            LineNode::new(Point2::new(-434.0,   76.0)),
            LineNode::new(Point2::new(-224.0, -116.0)),
        ];
        expected_nodes[0].promote_to_stop();
        expected_nodes[1].promote_to_stop();
        expected_nodes[2].promote_to_stop();
        expected_nodes[3].promote_to_stop();
        expected_nodes[4].promote_to_stop();
        assert_abs_diff_eq!(*route.freeze_nodes(), expected_nodes, epsilon = 0.01);
    }
}
