use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
pub struct Acceleration {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
