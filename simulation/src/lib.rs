mod color;
mod direction;
pub mod line;
mod node;
mod station;
mod train;

pub use crate::color::Color;
pub use crate::direction::{Direction, Directions};
pub use crate::line::{Line, LineGroup};
pub use crate::node::{Kind as NodeKind, Node};
pub use crate::station::Station;
pub use crate::train::Train;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    pub use crate::node::fixtures as nodes;
    pub use crate::station::fixtures as stations;
    pub use crate::train::fixtures as trains;
}
