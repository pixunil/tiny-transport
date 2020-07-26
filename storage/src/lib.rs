#![allow(clippy::zero_prefixed_literal)]

mod dataset;
mod line;
mod node;
mod schedule;
mod station;
mod train;

pub use crate::dataset::Dataset;
pub use crate::line::Line;
pub use crate::node::{Kind as NodeKind, Node};
pub use crate::schedule::Schedule;
pub use crate::station::Station;
pub use crate::train::Train;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures {
    pub use crate::dataset::fixtures as datasets;
    pub use crate::line::fixtures as lines;
    pub use crate::node::fixtures as nodes;
    pub use crate::schedule::fixtures as schedules;
    pub use crate::station::fixtures as stations;
    pub use crate::train::fixtures as trains;

    #[macro_export]
    macro_rules! fixtures_with_ids {
        (@ids { $( $name:ident ),* $(,)? }) => {{
            vec![ $( stringify!($name) ),* ]
                .into_iter()
                .enumerate()
                .map(|(i, identifier)| (identifier, i))
                .collect::<std::collections::HashMap<_, _>>()
        }};
        ($kind:ident :: { $( $name:ident ),* $(,)? }) => {{
            let fixtures = vec![ $( $crate::fixtures::$kind::$name() ),* ];
            (fixtures, fixtures_with_ids!(@ids { $($name),* }))
        }};
        (simulation :: $kind:ident :: { $( $name:ident ),* $(,)? } with Rc) => {{
            let fixtures = vec![ $( std::rc::Rc::new(simulation::fixtures::$kind::$name()) ),* ];
            (fixtures, fixtures_with_ids!(@ids { $($name),* }))
        }};
    }
}
