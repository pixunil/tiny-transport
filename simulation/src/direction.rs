use serde_derive::{Deserialize, Serialize};

use crate::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Upstream,
    Downstream,
}

impl Direction {
    pub(crate) fn start(self, len: usize) -> usize {
        match self {
            Self::Upstream => 0,
            Self::Downstream => len - 1,
        }
    }

    pub(crate) fn find_next(self, current: usize, nodes: &[Node]) -> Option<usize> {
        match self {
            Self::Upstream => nodes[current + 1..]
                .iter()
                .position(|node| node.allows(self))
                .map(|position| position + current + 1),
            Self::Downstream => nodes[..current].iter().rposition(|node| node.allows(self)),
        }
    }

    pub(crate) fn find_previous(self, current: usize, nodes: &[Node]) -> Option<usize> {
        match self {
            Self::Upstream => nodes[..current].iter().rposition(|node| node.allows(self)),
            Self::Downstream => nodes[current + 1..]
                .iter()
                .position(|node| node.allows(self))
                .map(|position| position + current + 1),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Directions {
    Both,
    UpstreamOnly,
    DownstreamOnly,
}

impl Directions {
    pub fn allows(self, direction: Direction) -> bool {
        match self {
            Self::Both => true,
            Self::UpstreamOnly => direction == Direction::Upstream,
            Self::DownstreamOnly => direction == Direction::Downstream,
        }
    }
}

impl From<Direction> for Directions {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Upstream => Self::UpstreamOnly,
            Direction::Downstream => Self::DownstreamOnly,
        }
    }
}
