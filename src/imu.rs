use anyhow::Result;
use schema::Acceleration as AccelerationData;
use serde::Serialize;

/// 3軸の加速度を返す
#[derive(Serialize, PartialEq, Clone, Debug)]
#[serde(transparent)]
pub struct Acceleration(pub AccelerationData);

impl Acceleration {
    pub fn new(x: f64, y: f64, z: f64) -> Acceleration {
        Acceleration(AccelerationData { x, y, z })
    }
}

pub trait Accelerometer {
    fn fetch(&mut self) -> Result<Acceleration>;
}
