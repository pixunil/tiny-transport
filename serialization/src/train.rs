use simulation::Direction;

#[derive(Debug, Serialize, Deserialize)]
pub struct Train {
    direction: Direction,
    arrivals: Vec<u32>,
    departures: Vec<u32>,
}

impl Train {
    pub fn new(direction: Direction, arrivals: Vec<u32>, departures: Vec<u32>) -> Train {
        Train {
            direction,
            arrivals,
            departures,
        }
    }

    pub fn unfreeze(self) -> simulation::Train {
        simulation::Train::new(self.direction, self.arrivals, self.departures)
    }
}
