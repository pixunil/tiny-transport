use std::collections::hash_map::{Entry, HashMap};
use std::rc::Rc;

use crate::location::{Location, LocationId};

#[derive(Debug)]
pub(crate) struct Linearizer {
    ids: HashMap<LocationId, usize>,
    locations: Vec<Rc<Location>>,
}

impl Linearizer {
    pub(crate) fn new() -> Self {
        Self {
            ids: HashMap::new(),
            locations: Vec::new(),
        }
    }

    pub(crate) fn retrieve(&mut self, location: &Rc<Location>) -> usize {
        match self.ids.entry(location.id()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                self.locations.push(location.clone());
                *entry.insert(self.locations.len() - 1)
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn location_ids(&self) -> HashMap<String, usize> {
        self.ids
            .iter()
            .map(|(identifier, id)| (format!("{}", identifier), *id))
            .collect()
    }
}

impl IntoIterator for Linearizer {
    type Item = Rc<Location>;
    type IntoIter = <Vec<Rc<Location>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.locations.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use itertools::assert_equal;

    use super::*;
    use crate::location::fixtures::locations;

    #[test]
    fn test_retrieve() {
        let mut linearizer = Linearizer::new();
        assert_eq!(linearizer.retrieve(&Rc::new(locations::hauptbahnhof())), 0);
        assert_eq!(linearizer.retrieve(&Rc::new(locations::friedrichstr())), 1);
        assert_eq!(linearizer.retrieve(&Rc::new(locations::hauptbahnhof())), 0);
    }

    #[test]
    fn test_into_vec() {
        let mut linearizer = Linearizer::new();
        linearizer.retrieve(&Rc::new(locations::hauptbahnhof()));
        linearizer.retrieve(&Rc::new(locations::friedrichstr()));
        assert_equal(
            linearizer,
            vec![
                Rc::new(locations::hauptbahnhof()),
                Rc::new(locations::friedrichstr()),
            ],
        );
    }
}
