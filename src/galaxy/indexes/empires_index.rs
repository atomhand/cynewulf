use crate::prelude::*;
use bevy::prelude::*;

pub fn update_empire_index_system(
    stars_query: Query<(Entity, &StarClaim)>,
    colonies_query: Query<(Entity, &Colony)>,
    mut empire_query: Query<(&mut EmpireIndex, Entity)>,
) {
    for (mut index, empire) in empire_query.iter_mut() {
        index.colonies.clear();
        index.systems.clear();
        index.population = 0;

        // These need to be ordered? so lists are consistent

        let mut tmp_systems = Vec::<(Entity, &StarClaim)>::new();
        let mut tmp_colonies = Vec::<(Entity, &Colony)>::new();

        for (entity, starclaim) in &stars_query {
            if let Some(owner) = starclaim.owner {
                if owner == empire {
                    //index.systems.push(entity);
                    tmp_systems.push((entity, starclaim));
                }
            }
        }
        for (entity, colony) in &colonies_query {
            if colony.owner == empire {
                //index.colonies.push(entity);
                tmp_colonies.push((entity, colony));
                index.population += colony.population.val();
            }
        }

        tmp_systems.sort_by(|(_, a), (_, b)| a.claimed_tick.cmp(&b.claimed_tick));
        tmp_colonies.sort_by(|(_, a), (_, b)| a.claimed_tick.cmp(&b.claimed_tick));

        index.systems.extend(tmp_systems.into_iter().map(|x| x.0));
        index.colonies.extend(tmp_colonies.into_iter().map(|x| x.0));
    }
}

#[derive(Component, Default)]
pub struct EmpireIndex {
    pub colonies: Vec<Entity>,
    pub systems: Vec<Entity>,
    pub population: i64,
}
