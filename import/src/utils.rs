pub mod deserialize;
mod dataset;

pub use serde_derive::Deserialize;

pub use dataset::Dataset;

pub type Id = String;

#[macro_export]
macro_rules! create_id_type {
    ($name:ident) => (
        use std::fmt;

        use serde_derive::Deserialize;

        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
        pub(crate) struct $name(String);

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(formatter)
            }
        }
    );
}
