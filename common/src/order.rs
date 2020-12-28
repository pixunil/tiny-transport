use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum Order {
    Forward,
    Backward,
}
