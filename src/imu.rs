use anyhow::Result;
use serde::Serialize;

/// 3軸の加速度を返す
#[derive(Serialize, PartialEq, Clone, Debug)]
pub struct Acceleration {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub trait Accelerometer {
    fn fetch(&mut self) -> Result<Acceleration>;
}
