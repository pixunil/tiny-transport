use std::collections::HashMap;
use std::error::Error;

use super::smoother::Mode;
use super::{Buffer, Segmenter, ShapeId, ShapeRecord, Shapes};
use crate::utils::Action;
use crate::utils::Dataset;

pub(crate) struct Importer;

impl Importer {
    pub(crate) fn import(dataset: &mut impl Dataset, mode: Mode) -> Result<Shapes, Box<dyn Error>> {
        let buffers =
            Self::read_into_buffers(dataset).map(|buffers| Self::smooth(buffers, mode))?;
        Ok(Self::segment(buffers))
    }

    fn read_into_buffers(
        dataset: &mut impl Dataset,
    ) -> Result<HashMap<ShapeId, Buffer>, Box<dyn Error>> {
        let mut buffers = HashMap::new();

        let action = Action::start("Importing shapes");
        for result in action.read_csv(dataset, "shapes.txt")? {
            let record: ShapeRecord = result?;
            record.import(&mut buffers);
        }
        action.complete(&format!("Imported {} shapes", buffers.len()));
        Ok(buffers)
    }

    fn smooth(mut buffers: HashMap<ShapeId, Buffer>, mode: Mode) -> HashMap<ShapeId, Buffer> {
        if mode != Mode::Off {
            let mut action = Action::start("Smoothing shapes");
            buffers = action
                .wrap_iter(buffers)
                .map(|(id, buffer)| (id, mode.smooth(buffer)))
                .collect();
            action.complete("Smoothed shapes");
        }
        buffers
    }

    fn segment(buffers: HashMap<ShapeId, Buffer>) -> Shapes {
        let mut segmenter = Segmenter::new();
        let mut action = Action::start("Segmenting shapes");
        for (id, buffer) in action.wrap_iter(buffers) {
            segmenter.segment(id, buffer);
        }
        let shapes = segmenter.finish();
        action.complete(&format!(
            "Segmented shapes into {} segments",
            shapes.segment_count()
        ));
        shapes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset;
    use crate::fixtures::shape_buffers;
    use test_utils::{assert_eq_alternate, map};

    #[test]
    fn test_read_into_buffers() {
        let mut dataset = dataset!(
            shapes:
                shape_id,                                       shape_pt_lat, shape_pt_lon;
                "u4::nollendorfplatz_innsbrucker_platz",        52.500,       13.354;
                "u4::innsbrucker_platz_nollendorfplatz",        52.478,       13.343;
                "u4::nollendorfplatz_innsbrucker_platz",        52.496,       13.343;
                "u4::nollendorfplatz_innsbrucker_platz",        52.489,       13.340;
                "u4::innsbrucker_platz_nollendorfplatz",        52.483,       13.342;
                "u4::innsbrucker_platz_nollendorfplatz",        52.489,       13.340;
                "u4::innsbrucker_platz_nollendorfplatz",        52.496,       13.343;
                "u4::nollendorfplatz_innsbrucker_platz",        52.483,       13.342;
                "u4::nollendorfplatz_innsbrucker_platz",        52.478,       13.343;
                "u4::innsbrucker_platz_nollendorfplatz",        52.500,       13.354
        );
        assert_eq_alternate!(
            Importer::read_into_buffers(&mut dataset).unwrap(),
            map! {
                "u4::nollendorfplatz_innsbrucker_platz"
                    => shape_buffers::u4::nollendorfplatz_innsbrucker_platz(),
                "u4::innsbrucker_platz_nollendorfplatz"
                    => shape_buffers::u4::innsbrucker_platz_nollendorfplatz(),
            }
        );
    }
}
