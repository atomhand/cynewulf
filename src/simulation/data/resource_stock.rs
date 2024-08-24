use crate::util::number::IFraction;
use crate::prelude::*;

pub struct ResourceStock {
    pub stock : i64,
    pub stock_frac : i64, // 3650ths of a stock
    pub change_per_decade : i64,
}

impl Default for ResourceStock {
    fn default() -> Self {
        Self {
            stock : 0,
            stock_frac : 0,
            change_per_decade : 0
        }
    }
}

impl std::string::ToString for ResourceStock {
    fn to_string(&self) -> String {
        self.stock.format_number()
    }
}

impl ResourceStock {
    pub fn new(val : i64) -> Self {
        ResourceStock {
            stock : val,
            stock_frac : 0,
            change_per_decade : 0
        }
    }
    pub fn set(&mut self, val : i64) {
        self.stock = val;
        self.stock_frac = 0;
    }

    pub fn increment_daily(&mut self) {
        let fr = self.stock_frac + self.change_per_decade;

        self.stock_frac = fr % 3650;
        self.stock = self.stock + fr / 3650;

    }

    pub fn set_change_per_decade(&mut self, rate : i64) {
        self.change_per_decade = rate;
    }

}