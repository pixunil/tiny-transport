use na::Point2;

#[derive(Debug)]
pub struct Station {
    id: usize,
    position: Point2<f32>,
    name: String,
}

impl Station {
    fn new(id: usize, x: f32, y: f32, name: String) -> Station {
        Station {
            id: id,
            position: Point2::new(x, y),
            name: name,
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

    pub fn from_json(id: usize, json: &serde_json::Value) -> Station {
        let x = json["x"].as_f64().unwrap() as f32;
        let y = json["y"].as_f64().unwrap() as f32;
        let name = json["name"].as_str().unwrap().into();
        Station::new(id, x, y, name)
    }

    pub fn position_buffer_data(&self) -> impl Iterator<Item=f32> + '_ {
        self.position.iter().cloned()
    }
}
