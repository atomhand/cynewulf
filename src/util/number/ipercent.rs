use std::ops;

#[derive(Copy, Clone)]
pub struct IPercent {
    // SCALE : 0 to 1000
    value: i32,
}

use std::fmt;

impl fmt::Display for IPercent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}%", self.value / 10, self.value % 10)
    }
}

impl IPercent {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

impl ops::Mul<IPercent> for i64 {
    type Output = i64;

    fn mul(self, _rhs: IPercent) -> i64 {
        (_rhs.value as i64 * self) / 1000
    }
}
impl ops::Mul<IPercent> for u64 {
    type Output = u64;

    fn mul(self, _rhs: IPercent) -> u64 {
        (_rhs.value as u64 * self) / 1000
    }
}
