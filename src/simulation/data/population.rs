use crate::prelude::*;
use super::resource_stock::ResourceStock;

/* 
impl PopulationNumber {
    pub const ZERO : Self = PopulationNumber {
        raw : ResourceStock {
            stock : 0,
            stock_frac : 0,
            change_per_decade : 0
        }
    };

    pub fn new(raw : i64) -> Self {
        PopulationNumber {
            raw : ResourceStock::new(raw)
        }
    }

    pub fn actual(&self) -> u64 {
        self.raw.stock
    }
}
impl std::string::ToString for PopulationNumber {
    fn to_string(&self) -> String {
        self.raw.format_number()
    }
}
*/
pub struct Population(ResourceStock);

impl std::string::ToString for Population {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Population {
    pub fn new(raw : i64) -> Self {
        Self(ResourceStock::new(raw))
    }
    pub fn add(&mut self, val : i64) {
        self.0.stock += val;
    }
    pub fn set(&mut self, val : i64) {
        self.0.stock = val;
    }
    pub fn val(&self) -> i64 {
        self.0.stock
    }
    pub fn decade_birth_rate(&self) -> i64 {
        self.0.stock * 10 / 50
    }

    pub fn increment_daily(&mut self, planet : &Planet) {
        let birth = self.decade_birth_rate();
        let inverse_death_rate = 100 * planet.get_population_support() as i64 / self.0.stock;
        let death = self.0.stock * 10 / inverse_death_rate;

        self.0.set_change_per_decade(birth-death);
        self.0.increment_daily();
    }
}
