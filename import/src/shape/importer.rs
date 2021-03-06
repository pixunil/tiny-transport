use std::collections::HashMap;
use std::error::Error;

use super::smoother::Mode;
use super::{Shape, ShapeId, ShapeRecord};
use crate::utils::Action;
use crate::utils::Dataset;

pub(crate) struct Importer;

impl Importer {
    pub(crate) fn import(
        dataset: &mut impl Dataset,
        mode: Mode,
    ) -> Result<HashMap<ShapeId, Shape>, Box<dyn Error>> {
        let mut shapes = HashMap::new();

        let action = Action::start("Importing shapes");
        for result in action.read_csv(dataset, "shapes.txt")? {
            let record: ShapeRecord = result?;
            record.import(&mut shapes);
        }
        action.complete(&format!("Imported {} shapes", shapes.len()));

        if mode != Mode::Off {
            let mut action = Action::start("Smoothing shapes");
            shapes = action
                .wrap_iter(shapes)
                .map(|(id, shape)| (id, mode.smooth(shape)))
                .collect();
            action.complete("Smoothed shapes");
        }
        Ok(shapes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset;
    use crate::fixtures::shapes;
    use test_utils::{assert_eq_alternate, map};

    #[test]
    fn test_from_csv() {
        let mut dataset = dataset!(
            shapes:
                shape_id, shape_pt_lat, shape_pt_lon;
                1,        52.500,       13.354;
                2,        52.478,       13.343;
                1,        52.496,       13.343;
                1,        52.489,       13.340;
                2,        52.483,       13.342;
                2,        52.489,       13.340;
                2,        52.496,       13.343;
                1,        52.483,       13.342;
                1,        52.478,       13.343;
                2,        52.500,       13.354
        );

        assert_eq_alternate!(
            Importer::import(&mut dataset, Mode::Full).unwrap(),
            map! {
                "1" => shapes::u4::nollendorfplatz_innsbrucker_platz(),
                "2" => shapes::u4::innsbrucker_platz_nollendorfplatz(),
            }
        );
    }
}
