mod kind;
mod line;

pub use kind::Kind;
pub use line::Line;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    pub use super::line::fixtures as lines;
}
