use bevy::prelude::*;
use crate::prelude::*;



#[derive(Component)]
pub struct Colony {
    pub owner : Entity,
    pub claimed_tick : i64,
    pub population : Population,
    pub economy : Economy
}

impl Colony {
    // THESE COULD USE THE FRACTIONAL SYSTEM
    // IT DOESN't ACTUALLY MATTER THAT MUCH THO, because it's quite intentional that a planet's population is Quite Large before it starts launching colony ships
    pub fn get_daily_colonists(&self) -> i64 {
        self.population.val() / 30000000 // 30e6
    }
    pub fn get_daily_colony_ship_construction(&self) -> i64 {
        self.economy.advanced_infra / 3000000
    }
}


#[derive(Component)]
pub struct StarClaim {
    pub owner : Option<Entity>,
    pub claimed_tick : i64
}