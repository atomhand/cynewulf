use bevy::prelude::*;
use crate::prelude::*;
use bevy::ecs::batching::BatchingStrategy;
use rand::prelude::*;

use super::navigation::{
    NavPosition,
    Plan,
    Navigator,
    Action
};

pub fn nav_find_colony_target_system(
    mut nav_query : Query<(&mut NavPosition, &mut Navigator, &Fleet)>,
    system_query : Query<(&Star,&StarClaim),Without<Navigator>>,
    planet_query : Query<(&Planet,Entity), (Without<Colony>,Without<Navigator>)>,
    hypernet : Res<Hypernet>,
){

    nav_query
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::fixed(32))
        .for_each(|(nav_pos,mut nav, _fleet)|
    {
        let mut rng = rand::thread_rng();
        let Action::Idle = nav.action else { return; };
        if nav.plan_queue.len() > 0 { return; }

        let mut best_option : Option<(u32,Entity)> = None;
        let mut best_dist = usize::MAX;
        for(star,starclaim) in system_query.iter() {
            if let Some(_star_owner) = starclaim.owner {
                continue;
                //if star_owner != fleet.owner { continue; }
            }
    
            let Some((_planet,planet_entity)) = star.orbiters.iter().filter_map(|x| planet_query.get(*x).ok()).choose(&mut rng) else { continue; };
    
            if let Some(path) = hypernet.find_path(nav_pos.root_system, star.node_id) {
                let d = path.edges.len();
    
                if d < best_dist {
                    best_dist = d;
                    best_option = Some((star.node_id,planet_entity));
                }
            }
        }

        if let Some((star_id,planet_entity)) = best_option {
            nav.plan_queue.push(Plan::Colonise(planet_entity));
            nav.plan_queue.push(Plan::ReachSystem(star_id));
        }
    });
}

#[derive(Event)]
pub struct ColonisePlanetEvent {
    pub planet_entity : Entity,
    pub colony_fleet : Entity
}

pub fn process_colonise_events  (
    mut planet_query : Query<(&Planet,&Parent,Option<&mut Colony>)>,
    mut fleet_query : Query<(&Fleet, &mut Navigator), (Without<Colony>, Without<StarClaim>)>,
    mut star_query: Query<&mut StarClaim, (With<Star>,Without<Colony>, Without<Fleet>)>,
    mut ev_colonise : EventReader<ColonisePlanetEvent>,
    mut commands : Commands
) {
    for ev in ev_colonise.read() {
        let Ok((fleet,mut nav)) = fleet_query.get_mut(ev.colony_fleet) else { continue; };
        let Ok((_planet, parent,colony)) = planet_query.get_mut(ev.planet_entity) else { continue; };
        let Ok(mut star_claim) = star_query.get_mut(parent.get()) else { continue; };

        if let Some(owner) = star_claim.owner {
            if owner != fleet.owner {
                info!("colonisation failed: System is already claimed by someone else");
                // reset the fleet
                nav.action = Action::Idle;
                nav.plan_queue.clear();
                continue;
            }
        } else {
            star_claim.owner = Some(fleet.owner);
        }

        commands.entity(ev.colony_fleet).despawn();

        let colony_population = 10000;

        if let Some(mut colony) = colony {
            // could check that the existing colony has the right owner...
            colony.population.add(colony_population);
        } else {
            commands.entity(ev.planet_entity)
                .insert(Colony {
                    owner : fleet.owner,
                    population : Population::new(colony_population),
                    economy : Economy::new()
                });
        }
    }
}