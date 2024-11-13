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
            }
        }
    }
}

#[derive(Component,Default)]
pub struct EmpireIndex {
    pub colonies : Vec<Entity>,
    pub systems : Vec<Entity>
}