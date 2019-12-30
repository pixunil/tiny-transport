use std::collections::HashMap;

use serde_derive::Deserialize;

use super::{Shape, ShapeId, transform};

#[derive(Debug, Deserialize)]
pub(super) struct ShapeRecord {
    shape_id: ShapeId,
    shape_pt_lat: f32,
    shape_pt_lon: f32,
}

impl ShapeRecord {
    pub(super) fn import(self, shapes: &mut HashMap<ShapeId, Shape>) {
        let waypoint = transform(self.shape_pt_lat, self.shape_pt_lon);
        shapes.entry(self.shape_id)
            .or_insert_with(Vec::new)
            .push(waypoint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::map;

    #[test]
    fn test_import() {
        let mut shapes = HashMap::new();
        let record = ShapeRecord {
            shape_id: "1".into(),
            shape_pt_lat: 52.526,
            shape_pt_lon: 13.369,
        };
        record.import(&mut shapes);
        assert_eq!(shapes, map! {
            "1" => vec![transform(52.526, 13.369)],
        });
    }
}
