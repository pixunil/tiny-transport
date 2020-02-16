use na::Point2;

#[derive(Debug)]
pub struct Station {
    position: Point2<f32>,
    name: String,
}

impl Station {
    pub fn new(position: Point2<f32>, name: String) -> Station {
        Station { position, name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn contains(&self, position: Point2<f32>) -> bool {
        na::distance(&self.position, &position) <= 90.0
    }

    pub fn fill_vertice_buffer(&self, buffer: &mut Vec<f32>) {
        buffer.extend(self.position.iter())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    #[macro_export]
    macro_rules! station {
        (main_station) => (
            Station::new(Point2::new(200.0, 100.0), "Main Station".to_string())
        );
    }

    #[test]
    fn test_station_contains_friedrich_street() {
        let station = station!(main_station);
        assert!(station.contains(Point2::new(200.0, 100.0)));
    }

    #[test]
    fn test_station_contains_border() {
        let station = station!(main_station);
        assert!(station.contains(Point2::new(205.0, 100.0)));
        assert!(station.contains(Point2::new(200.0, 95.0)));
        assert!(station.contains(Point2::new(195.0, 100.0)));
        assert!(station.contains(Point2::new(200.0, 105.0)));
    }

    #[test]
    fn test_station_excludes_outside() {
        let station = station!(main_station);
        assert!(!station.contains(Point2::new(110.0, 10.0)));
        assert!(!station.contains(Point2::new(290.0, 190.0)));
        assert!(!station.contains(Point2::new(290.5, 100.0)));
    }

    #[test]
    fn test_station_vertices() {
        let station = station!(main_station);
        let mut buffer = Vec::new();
        station.fill_vertice_buffer(&mut buffer);
        assert_relative_eq!(*buffer, [200.0, 100.0])
    }
}
