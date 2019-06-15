use std::rc::Rc;

use simulation::Color;
use super::train::Train;

type StationBinding = [Rc<simulation::Station>];

#[derive(Debug, Serialize, Deserialize)]
pub struct Line {
    name: String,
    stops: Vec<usize>,
    trains: Vec<Train>,
}

impl Line {
    pub fn new(name: String, stops: Vec<usize>, trains: Vec<Train>) -> Line {
        Line {
            name,
            stops,
            trains,
        }
    }

    fn unfreeze(self, stations: &StationBinding, color: &Color) -> simulation::Line {
        let stations = self.stops.into_iter()
            .map(|stop| stations[stop].clone())
            .collect::<Vec<_>>();

        let trains = self.trains.into_iter()
            .map(|train| train.unfreeze())
            .collect();

        simulation::Line::new(self.name, color.clone(), stations, trains)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LineGroup {
    color: Color,
    lines: Vec<Line>,
}

impl LineGroup {
    pub fn new(color: Color, lines: Vec<Line>) -> LineGroup {
        LineGroup {
            color,
            lines,
        }
    }

    pub fn unfreeze(self, stations: &StationBinding) -> simulation::LineGroup {
        let color = self.color;
        let lines = self.lines.into_iter()
            .map(|line| line.unfreeze(stations, &color))
            .collect();
        simulation::LineGroup::new(color, lines)
    }
}
