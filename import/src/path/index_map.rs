use std::collections::HashMap;

use crate::location::Linearizer;
use crate::path::Segment;
use itertools::Itertools;

#[derive(Debug)]
pub(crate) struct IndexMap {
    indices: HashMap<usize, usize>,
}

impl IndexMap {
    pub(crate) fn new() -> Self {
        Self {
            indices: HashMap::new(),
        }
    }

    pub(super) fn retrieve(&mut self, index: usize) -> usize {
        let len = self.indices.len();
        *self.indices.entry(index).or_insert(len)
    }

    pub(crate) fn store_segments(
        self,
        segments: &[Segment],
        linearizer: &mut Linearizer,
    ) -> Vec<storage::Segment> {
        self.indices
            .into_iter()
            .sorted_by_key(|&(_, storage_index)| storage_index)
            .map(|(index, _)| segments[index].store(linearizer))
            .collect()
    }
}

#[cfg(tests)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieve() {
        let mut index_map = IndexMap::new();
        assert_eq!(index_map.retrieve(1), 0);
        assert_eq!(index_map.retrieve(4), 1);
        assert_eq!(index_map.retrieve(1), 0);
        assert_eq!(index_map.retrieve(2), 2);
    }

    #[test]
    fn test_store_segments() {
        let mut index_map = IndexMap::new();
        index_map.retrieve(1);
        todo!();
    }
}
