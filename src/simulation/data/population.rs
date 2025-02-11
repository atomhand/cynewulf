use super::dynamic_stock::DynamicStock;
use crate::prelude::*;

pub struct Population {
    pop: DynamicStock,
    planet_capacity: i64,
    birth_rate: IFraction,
    death_rate: IFraction,
    births: i64,
    deaths: i64,
}

impl std::string::ToString for Population {
    fn to_string(&self) -> String {
        self.pop.to_string()
    }
}

impl Population {
    pub fn new(raw: i64) -> Self {
        Self {
            pop: DynamicStock::new(raw),
            planet_capacity: 0,
            birth_rate: IFraction::new(0, 1),
            death_rate: IFraction::new(0, 1),
            births: 0,
            deaths: 0,
        }
    }
    pub fn add(&mut self, val: i64) {
        self.pop.stock += val;
    }
    pub fn set(&mut self, val: i64) {
        self.pop.stock = val;
    }
    pub fn val(&self) -> i64 {
        self.pop.stock
    }
    pub fn decade_birth_rate(&self) -> i64 {
        (self.pop.stock * 10) / 50
    }

    pub fn details(&self) -> String {
        format!("Capacity: {}\nBirth Rate: {:<7} | Death Rate: {:<7}\nBirths: {:<7} | Deaths: {:<7} | Net: {:<7}",
            self.planet_capacity.format_big_number(),
            self.birth_rate.display_as_percent(),
            self.death_rate.display_as_percent(),
            self.births.format_big_number(),
            self.deaths.format_big_number(),
            (self.births-self.deaths).format_big_number())
    }

    pub fn increment_daily(&mut self, planet: &Planet) {
        self.planet_capacity = planet.get_population_support() as i64;
        self.birth_rate = IFraction::new(10, 50);

        let birth = self.pop.stock * IFraction::new(10, 50); //;self.decade_birth_rate();
        let mut death_rate = IFraction::new(self.pop.stock * 10, self.planet_capacity * 50); //IFraction = IFraction::new(self.pop.stock * 10, self.planet_capacity * 50);
        death_rate.brute_simplify();
        self.death_rate = death_rate;
        let death = self.pop.stock * death_rate;

        self.births = birth;
        self.deaths = death;

        self.pop.set_change_per_decade(birth - death);
        self.pop.increment_daily();
    }
}
