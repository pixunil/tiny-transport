pub(crate) mod deserialize;
mod dataset;

pub(crate) use dataset::Dataset;

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

#[cfg(test)]
#[macro_export]
macro_rules! map {
    () => (
        std::collections::HashMap::new()
    );
    ($($key:expr => $value:expr),* $(,)?) => ({
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key.into(), $value);
        )*
        map
    });
}
