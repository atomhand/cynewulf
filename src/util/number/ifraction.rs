use std::ops;

#[derive(Copy,Clone)]
pub struct IFraction {
    numerator : i64,
    denominator : i64
}

impl IFraction {
    pub fn new(numerator : i64, denominator : i64) -> Self {
        Self {
            numerator,
            denominator
        }
    }

    pub fn display_as_percent(&self) -> String {
        let whole = 100 * self.numerator / self.denominator;
        let frac = (10000 * self.numerator / self.denominator) - whole;

        format!("{}.{}%", whole, frac)
    }
}

impl ops::Mul<IFraction> for i64 {
    type Output = i64;

    fn mul(self, _rhs : IFraction) -> i64 {
        (_rhs.numerator as i64 * self) / _rhs.denominator
    }
}