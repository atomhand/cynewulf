
use bevy::prelude::*;
use rand::prelude::*;
use std::collections::HashSet;

use crate::galaxy::Description;

use crate::prelude::*;

// This should probably be in a different file..
pub fn finish_create_colony(
    mut empire_query : Query<&mut Empire, Without<Description>>,
    mut colony_query : Query<(&mut  Description,&Colony), Added<Colony>>,
    mut used_planet_names : ResMut<super::markov_chain::UsedPlanetNames>,
) {
    for (mut desc, colony) in colony_query.iter_mut() {
        let Ok(mut empire) = empire_query.get_mut(colony.owner) else { continue; };

        desc.name = empire.namegen.next(&mut used_planet_names).clone();
    }
}

pub fn place_star_empires(mut commands : Commands,
    mut star_query : Query<(Entity,&Star, &mut StarClaim)>,
    planet_query : Query<&Planet, Without<Star>>,
    mut used_planet_names : ResMut<super::markov_chain::UsedPlanetNames>,
    mut player_empire : ResMut<crate::galaxy::empire::PlayerEmpire>
) {
    let num_empires = 10;

    let mut claimed_systems : HashSet<Entity> = HashSet::new();

    let mut rng = thread_rng();

    for i in 0..num_empires {
        let mut best : Option<(Entity,Entity,f32)> = None;

        for (star_entity,star, _star_claim) in &star_query {
            if claimed_systems.contains(&star_entity) { continue; }

            for planet_entity in &star.orbiters {
                if let Ok(planet) = planet_query.get(*planet_entity) {

                    // TODO - Rate planets on a factor that matters..
                    let score = rng.gen_range(0.01..1.0) / planet.get_visual_radius();
                    if let Some((_,_,old_score)) = best {
                        if score > old_score {
                            best = Some((*planet_entity,star_entity,score));
                        }
                    } else {
                        best = Some((*planet_entity,star_entity,score));
                    }
                }
            }
        }

        if let Some((planet_entity,star_entity,score)) = best {
            if score > 0.0 {
                let new_empire = commands.spawn(Empire::random(&mut rng, &mut used_planet_names)).id();

                // ~~ temp
                // give the player an empire
                if player_empire.empire == None {
                    player_empire.empire = Some(new_empire);
                }

                let (_,star,mut star_claim) = star_query.get_mut(star_entity).unwrap();
                star_claim.owner = Some(new_empire);
                commands.entity(planet_entity).insert(Colony {
                    owner : new_empire,
                    population : Population::new(9e9 as i64),
                    economy : Economy::new()
                });

                claimed_systems.insert(star_entity);

                // TEMP - Spawn a fleet too for luck
                commands.spawn(crate::galaxy::fleet::FleetBundle::new(new_empire, star.node_id));
            }
        }
    }    
}