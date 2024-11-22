
use std::fmt;
use crate::util::number::IPercent;
use crate::prelude::*;

pub struct Economy {
    light_infra : i64,
    heavy_infra : i64,
    pub advanced_infra : i64,

    light_growth : i64,
    heavy_growth : i64,
    pub advanced_growth : i64,

    basic_workers : i64,
    advanced_workers : i64,

    // Input parameter to training_rate, representing the share of the population who have the potential to be advanced workers
    // Unit: Percentage points
    worker_potential : IPercent, 

    engineer_growth : i64,// Number of basic workers trained into advanced workers
    training_rate : IPercent, // effectiveness at which advanced workers train new engineers

    // Observation
    //  -- High worker potential always increases your max num of engineers and the speed at which you train engineers
    //  -- High training rate increases the speed at which you train engineers, but only increases max num of engineers if you are bottlenecked by retiring rate rather than worker potential
    // Ie. training rate may be a less important parameter for a civilization with high lifespans
    // Open question - is training rate an interesting enough parameter to bother with?

    // Rate at which advanced engineers retire (replaced by newborn basic workers).
    // Can be inherited from POpulation/death rate, once there is one
    // Unit: Percentage points
    retiring_rate : IPercent,

    light_output : i64,
    heavy_output : i64,
    advanced_output : i64,

    infra_depreciation_rate : IPercent,
    savings_rate : IPercent,

    basic_wage : i32,
    advanced_wage : i32,
}

impl Economy {
    pub fn new() -> Self {
        Self {
            light_infra : 1,
            heavy_infra : 1,
            advanced_infra : 1,

            light_growth : 0,
            heavy_growth : 0,
            advanced_growth : 0,

            basic_workers : 0,
            advanced_workers : 0,

            worker_potential : IPercent::new(0),

            engineer_growth : 0,
            training_rate : IPercent::new(0),

            retiring_rate : IPercent::new(0),

            light_output : 0,
            heavy_output : 0,
            advanced_output : 0,

            infra_depreciation_rate : IPercent::new(0),
            savings_rate : IPercent::new(0),

            basic_wage : 0,
            advanced_wage : 0
        }
    }


    pub fn update_dynamic_params(&mut self, population : i64) {
        self.savings_rate = IPercent::new(250);
        self.infra_depreciation_rate = IPercent::new(100);

        self.basic_wage = 100;
        self.advanced_wage = 500;

        self.light_output = (self.advanced_workers + self.basic_workers).isqrt() * self.light_infra.isqrt().max(1);
        self.heavy_output = (self.advanced_workers / 2 + self.basic_workers / 10).isqrt() * self.heavy_infra.isqrt();
        self.advanced_output = (self.advanced_workers / 10).isqrt() * self.advanced_infra.isqrt();


        self.worker_potential = IPercent::new(200);
        self.training_rate = IPercent::new(150);

        self.basic_workers = population - self.advanced_workers;

        let retirees = self.advanced_workers * self.retiring_rate;
        // part of the population who are potential to be promoted to engineers (advanced workers)
        let potential_engineers = population * self.worker_potential - (self.advanced_workers - retirees);
        if potential_engineers > 0 {
            self.engineer_growth = potential_engineers.isqrt() * (self.advanced_workers * self.training_rate).isqrt().max(1);
        } else {
            self.engineer_growth = 0;
        }

        self.light_growth = (self.light_output * self.savings_rate)
            - self.light_infra * self.infra_depreciation_rate;

        self.heavy_growth = (self.heavy_output * self.savings_rate)
            - self.heavy_infra * self.infra_depreciation_rate;

        self.advanced_growth = (self.advanced_output * self.savings_rate)
            - self.advanced_infra * self.infra_depreciation_rate;
    }

    pub fn update_stocks(&mut self) {
        let net_worker_promotion = i64::min(self.basic_workers,self.engineer_growth as i64 - self.advanced_workers * self.retiring_rate);

        self.advanced_workers = self.advanced_workers + net_worker_promotion;
        self.basic_workers -= net_worker_promotion;


        self.light_infra = (self.light_infra + self.light_growth).max(0);
        self.heavy_infra = (self.heavy_infra + self.heavy_growth).max(0);
        self.advanced_infra = (self.advanced_infra + self.advanced_growth).max(0);
    }
}

impl fmt::Display for Economy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Output: \n")?;
        write!(f, "   Light: {} \n", self.light_output.format_big_number())?;
        write!(f, "   Heavy: {} \n", self.heavy_output.format_big_number())?;
        write!(f, "   Advanced: {} \n", self.advanced_output.format_big_number())?;

        write!(f, "Infrastructure: \n")?;
        write!(f, "   Light: {} (+{}) \n", self.light_infra.format_big_number(), self.light_growth.format_big_number())?;
        write!(f, "   Heavy: {} (+{}) \n", self.heavy_infra.format_big_number(), self.heavy_growth.format_big_number())?;
        write!(f, "   Advanced: {} (+{}) \n", self.advanced_infra.format_big_number(), self.advanced_growth.format_big_number())?;


        write!(f, "\nDepreciation Rate:{} \n", self.infra_depreciation_rate)?;
        write!(f, "Savings Rate:{} \n", self.savings_rate)?;
        write!(f, "Basic Workers:{} \n", self.basic_workers.format_big_number())?;
        write!(f, "Engineers:{} (+{}) \n", self.advanced_workers.format_big_number(), self.engineer_growth.format_big_number())
    }
}