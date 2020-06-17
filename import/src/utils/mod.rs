mod action;
mod dataset;

pub(crate) use action::Action;
pub(crate) use dataset::Dataset;

#[macro_export]
macro_rules! create_id_type {
    ($name:ident) => {
        use serde_derive::Deserialize;

        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
        pub(crate) struct $name(String);

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(formatter)
            }
        }
    };
}
