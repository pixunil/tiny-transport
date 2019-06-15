use na::Point2;

#[derive(Debug)]
pub struct Station {
    id: usize,
    position: Point2<f32>,
    name: String,
}

impl Station {
    pub fn new(id: usize, position: Point2<f32>, name: String) -> Station {
        Station {
            id,
            position,
            name,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn position(&self) -> Point2<f32> {
        self.position.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn contains(&self, position: &Point2<f32>) -> bool {
        na::distance(&self.position, position) < 5.0
    }

    pub fn position_buffer_data(&self) -> impl Iterator<Item=f32> + '_ {
        self.position.iter().cloned()
    }
}
