use bevy::prelude::*;
use crate::prelude::*;

pub fn update_empire_index_system(
    stars_query : Query<(Entity,&StarClaim)>,
    colonies_query : Query<(Entity,&Colony)>,
    mut empire_query : Query<(&mut EmpireIndex,Entity)>
) {
    for (mut index,empire) in empire_query.iter_mut() {
        index.colonies.clear();
        index.systems.clear();
        index.population = 0;

        // These need to be ordered? so lists are consistent

        for (entity,starclaim) in &stars_query {
            if let Some(owner) = starclaim.owner {
                if owner == empire {
                    index.systems.push(entity);
                }
            }
        }
        for (entity,colony) in &colonies_query {
            if colony.owner == empire {
                index.colonies.push(entity);
                index.population += colony.population.val();
            }
        }
    }
}

#[derive(Component,Default)]
pub struct EmpireIndex {
    pub colonies : Vec<Entity>,
    pub systems : Vec<Entity>,
    pub population : i64
}