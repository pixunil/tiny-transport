use std::collections::HashMap;
use std::iter;
use std::rc::Rc;

use itertools::Itertools;
use ordered_float::NotNan;

use crate::coord::Point;
use crate::location::Location;
use crate::path::{Node, Segment, SegmentRef, SegmentedPath};
use crate::shape::{self, PointRef};

struct StopCandidate<'a> {
    pos: usize,
    point: &'a PointRef,
    distance: NotNan<f64>,
    location: Rc<Location>,
}

impl<'a> StopCandidate<'a> {
    fn distance(a: &Point, b: &Point) -> NotNan<f64> {
        let distance = na::distance(a, b);
        NotNan::new(distance).unwrap()
    }

    fn find_nearest(
        points: &'a [PointRef],
        lower: usize,
        upper: usize,
        location: Rc<Location>,
    ) -> Self {
        let (pos, point) = points[lower..upper]
            .iter()
            .enumerate()
            .min_by_key(|(_, point)| Self::distance(point.position(), &location.position()))
            .unwrap();
        Self {
            pos: pos + lower,
            point,
            distance: Self::distance(point.position(), &location.position()),
            location,
        }
    }

    fn distribute_across(points: &'a [PointRef], locations: &[Rc<Location>]) -> Vec<Self> {
        let mut candidates: Vec<Self> = Vec::with_capacity(locations.len());
        for (i, location) in locations.iter().enumerate() {
            let upper = points.len() + i - locations.len() + 1;
            let candidate_nearest = Self::find_nearest(&points, i, upper, Rc::clone(location));

            if candidates
                .last()
                .map_or(true, |last| last.pos < candidate_nearest.pos)
            {
                candidates.push(candidate_nearest);
                continue;
            }

            let (at, lower) = candidates
                .iter()
                .enumerate()
                .map(|(i, candidate)| (i + 1, candidate.pos + 1))
                .rfind(|&(at, lower)| {
                    let following = candidates.len() - at;
                    lower + following < candidate_nearest.pos
                })
                .unwrap_or((0, 0));
            let locations_brought_forward = candidates[at..]
                .iter()
                .map(|position| &position.location)
                .cloned()
                .collect::<Vec<_>>();
            let mut candidates_brought_forward = Self::distribute_across(
                &points[lower..candidate_nearest.pos],
                &locations_brought_forward,
            );
            for position in &mut candidates_brought_forward {
                position.pos += lower;
            }

            let candidate_behind = Self::find_nearest(
                &points,
                candidates.last().unwrap().pos + 1,
                upper,
                Rc::clone(location),
            );
            if candidate_nearest.total_difference(&candidates_brought_forward)
                <= candidate_behind.total_difference(&candidates[at..])
            {
                candidates.splice(at.., candidates_brought_forward);
                candidates.push(candidate_nearest);
            } else {
                candidates.push(candidate_behind);
            }
        }

        debug_assert!(candidates
            .iter()
            .tuple_windows()
            .all(|(a, b)| a.pos < b.pos));
        candidates
    }

    fn total_difference(&self, candidates: &[Self]) -> f64 {
        *self.distance
            + candidates
                .iter()
                .map(|candidate| *candidate.distance)
                .sum::<f64>()
    }

    fn accept(self, specifications: &mut [SegmentSpecification]) {
        // TODO: This is not optimal
        let specification = specifications
            .iter_mut()
            .find(|specification| {
                specification.segment.segment_index() == self.point.segment_index()
            })
            .unwrap();
        let location = specification
            .modifications
            .locations
            .insert(self.point.segment_pos(), self.location);
        debug_assert!(location.is_none());
    }
}

#[derive(Debug, PartialEq)]
struct SegmentSpecification<'a> {
    segment: &'a shape::SegmentRef,
    modifications: SegmentModifications,
}

impl<'a> SegmentSpecification<'a> {
    fn create_segment(&self, shape_segments: &[shape::Segment]) -> Segment {
        let last = shape_segments[self.segment.segment_index()]
            .iter()
            .next_back()
            .unwrap();
        let additional = iter::repeat(last).take(self.modifications.additional_count);
        let nodes = shape_segments[self.segment.segment_index()]
            .iter()
            .chain(additional)
            .enumerate()
            .map(|(segment_pos, point)| {
                let location = self.modifications.locations.get(&segment_pos);
                Node::new(point, location.cloned())
            })
            .collect::<Vec<_>>();
        debug_assert_eq!(
            nodes.iter().filter_map(Node::location).count(),
            self.modifications.locations.len()
        );
        Segment::new(nodes)
    }
}
impl<'a> From<&'a shape::SegmentRef> for SegmentSpecification<'a> {
    fn from(segment: &'a shape::SegmentRef) -> Self {
        Self {
            segment,
            modifications: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
struct SegmentModifications {
    locations: HashMap<usize, Rc<Location>>,
    additional_count: usize,
}

pub(crate) struct StopPlacer<'a> {
    shape_segments: &'a [shape::Segment],
    path_segments: Vec<Segment>,
    retrieval: HashMap<usize, Vec<(SegmentModifications, usize)>>,
}

impl<'a> StopPlacer<'a> {
    pub(crate) fn new(segments: &'a [shape::Segment]) -> Self {
        Self {
            shape_segments: segments,
            path_segments: Vec::new(),
            retrieval: HashMap::new(),
        }
    }

    pub(crate) fn place_stops(
        &mut self,
        shape: &shape::SegmentedShape,
        locations: &[Rc<Location>],
    ) -> SegmentedPath {
        let mut specifications = shape
            .segments()
            .iter()
            .map(SegmentSpecification::from)
            .collect::<Vec<_>>();

        let last_specification = specifications.last_mut().unwrap();
        let points = self.points_with_at_least(shape, locations.len(), last_specification);

        let candidates = StopCandidate::distribute_across(&points, locations);

        for candidate in candidates {
            candidate.accept(&mut specifications);
        }

        let mut segments = Vec::new();
        for specification in specifications {
            let segment_index = self
                .find_matching_variant(&specification)
                .unwrap_or_else(|| self.create_variant(&specification));

            segments.push(SegmentRef::new(
                segment_index,
                specification.segment.order(),
            ));
        }
        SegmentedPath::new(segments)
    }

    fn points_with_at_least(
        &self,
        shape: &shape::SegmentedShape,
        count: usize,
        last_specification: &mut SegmentSpecification,
    ) -> Vec<PointRef> {
        let mut points = shape.points(&self.shape_segments);
        if let Some(additional) = count.checked_sub(points.len()) {
            let last_point = points.last().cloned().unwrap();
            points.extend(
                (0..additional)
                    .into_iter()
                    .map(|offset| last_point.clone_with_offset(offset + 1)),
            );
            last_specification.modifications.additional_count = additional;
        }
        points
    }

    fn find_matching_variant(&self, specification: &SegmentSpecification) -> Option<usize> {
        let variants = self.retrieval.get(&specification.segment.segment_index())?;
        variants
            .iter()
            .find(|variant| variant.0 == specification.modifications)
            .map(|variant| variant.1)
    }

    fn create_variant(&mut self, specification: &SegmentSpecification) -> usize {
        let segment_index = self.path_segments.len();
        self.path_segments
            .push(specification.create_segment(&self.shape_segments));
        self.retrieval
            .entry(specification.segment.segment_index())
            .or_insert_with(Vec::new)
            .push((specification.modifications.clone(), segment_index));
        segment_index
    }

    pub(crate) fn finish(self) -> Vec<Segment> {
        self.path_segments
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::fixtures::{locations, path_segments, paths, shapes, stop_locations};
    use crate::shape::ShapeId;
    use common::{assert_eq_alternate, join};

    #[test]
    fn test_distance_to() {
        let distance = StopCandidate::distance(
            &path_segments::oranienburger_tor_friedrichstr().nodes()[3].position(),
            &locations::friedrichstr().position(),
        );
        assert_relative_eq!(distance.into_inner(), 67.86, epsilon = 0.01);
    }

    macro_rules! test_placer {
        ($line:ident :: $route:ident) => {{
            let shapes = shapes::by_id();
            let mut placer = StopPlacer::new(shapes.segments());
            let id = ShapeId::from(join!($line, $route));
            let path = placer.place_stops(&shapes[&id], &stop_locations::$line::$route());
            let actual_segments = placer.finish();
            let (segments, segment_ids) = paths::$line::segments();
            assert_eq_alternate!(
                path.nodes(&actual_segments).collect::<Vec<_>>(),
                paths::$line::$route(&segment_ids)
                    .nodes(&segments)
                    .collect::<Vec<_>>()
            );
        }};
    }

    #[test]
    fn test_different_direction_stops() {
        test_placer!(tram_12::oranienburger_tor_am_kupfergraben);
        test_placer!(tram_12::am_kupfergraben_oranienburger_tor);
    }

    #[test]
    fn test_circle() {
        test_placer!(s41::circle);
    }

    #[test]
    fn test_duplicated_stop() {
        test_placer!(bus_m41::anhalter_bahnhof_hauptbahnhof);
        test_placer!(bus_m41::hauptbahnhof_anhalter_bahnhof);
    }

    #[test]
    fn test_lasso() {
        test_placer!(bus_114::wannsee_heckeshorn_wannsee);
    }

    #[test]
    fn test_too_few_points() {
        test_placer!(tram_m10::strassmannstr_warschauer_str_too_few_points);
    }
}
