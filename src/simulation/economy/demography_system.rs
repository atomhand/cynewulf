use bevy::prelude::*;
use crate::prelude::*;

pub fn update_population(
    mut colony_query : Query<(&mut Colony, &Planet)>
) {
    for (mut colony,planet) in colony_query.iter_mut() {
        colony.population.increment_daily(planet);

        let pop = colony.population.val();
        colony.economy.update_dynamic_params(pop);
        colony.economy.update_stocks();
    }
}