use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component, Default)]
pub struct SystemIndex {
    pub population: i64,
}

pub fn update_system_index_system(
    mut stars_query: Query<(&Star, &mut SystemIndex)>,
    colonies_query: Query<&Colony>,
) {
    for (star, mut index) in stars_query.iter_mut() {
        index.population = 0;

        // These need to be ordered? so lists are consistent

        for colony in star
            .orbiters
            .iter()
            .filter_map(|x| colonies_query.get(*x).ok())
        {
            index.population += colony.population.val();
        }
    }
}
