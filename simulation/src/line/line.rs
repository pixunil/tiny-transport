use super::Kind;
use crate::color::Color;
use crate::path::{Segment, SegmentedPath};
use crate::train::Train;

#[derive(Debug, PartialEq)]
pub struct Line {
    name: String,
    color: Color,
    kind: Kind,
    path: SegmentedPath,
    trains: Vec<Train>,
}

impl Line {
    pub fn new(
        name: String,
        color: Color,
        kind: Kind,
        path: SegmentedPath,
        trains: Vec<Train>,
    ) -> Line {
        Line {
            name,
            color,
            kind,
            path,
            trains,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn path(&self) -> &SegmentedPath {
        &self.path
    }

    pub fn fill_color_buffer(&self, colors: &mut Vec<f32>) {
        colors.extend(self.color.iter().map(|component| component as f32 / 255.0));
    }

    pub fn active_trains(&self) -> impl Iterator<Item = &Train> {
        self.trains.iter().filter(|train| train.is_active())
    }

    pub fn update(&mut self, segments: &[Segment], time_passed: u32) {
        let nodes = self.path.nodes(segments);
        for train in &mut self.trains {
            train.update(time_passed, &nodes);
        }
    }
}

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    use std::ops::Index;

    use super::*;
    use crate::fixtures::{paths, trains};
    use common::time;

    macro_rules! lines {
        (@trains $line:ident, $route:ident, [ $( $( $(:)? $time:literal )* ),* ]) => {
            $( trains::$line::$route(time!($($time),*)) ),*
        };
        ($($line:ident: $name:literal, $kind:ident, $route:ident, $times:tt);* $(;)?) => {
            $(
                pub fn $line<'a>(
                    segment_ids: &impl Index<&'a str, Output = usize>,
                ) -> Line {
                    Line {
                        name: $name.to_string(),
                        color: Kind::$kind.color(),
                        kind: Kind::$kind,
                        path: paths::$line::$route(segment_ids),
                        trains: vec![
                            lines!(@trains $line, $route, $times),
                        ],
                    }
                }
            )*
        };
    }

    lines! {
        s3:                 "S3",           SuburbanRailway,
            hackescher_markt_bellevue, [7:24:54];
        u6:                 "U6",           UrbanRailway,
            naturkundemuseum_franzoesische_str, [5:55:40];
        tram_12:            "12",           Tram,
            oranienburger_tor_am_kupfergraben, [9:01:40];
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::fixtures::{lines, paths};
    use common::time;

    #[test]
    fn test_getters() {
        let (_, segment_ids) = paths::tram_12::segments();
        let line = lines::tram_12(&segment_ids);
        assert_eq!(line.name(), "12");
        assert_eq!(line.kind(), Kind::Tram);
        let expected = &paths::tram_12::oranienburger_tor_am_kupfergraben(&segment_ids);
        assert_eq!(line.path(), expected);
    }

    #[test]
    fn test_fill_color_buffer() {
        let (_, segment_ids) = paths::tram_12::segments();
        let line = lines::tram_12(&segment_ids);
        let mut colors = Vec::new();
        line.fill_color_buffer(&mut colors);
        assert_relative_eq!(*colors, [0.8, 0.04, 0.13], epsilon = 0.01);
    }

    #[test]
    #[ignore]
    fn test_active_trains() {
        let (segments, segment_ids) = paths::tram_12::segments();
        let mut line = lines::tram_12(&segment_ids);
        assert_eq!(line.active_trains().count(), 0);
        line.update(&segments, time!(8:33:40));
        assert_eq!(line.active_trains().count(), 0);
        line.update(&segments, time!(0:01:00));
        assert_eq!(line.active_trains().count(), 1);
        line.update(&segments, time!(0:06:00));
        assert_eq!(line.active_trains().count(), 0);
        line.update(&segments, time!(0:22:00));
        assert_eq!(line.active_trains().count(), 1);
    }
}
