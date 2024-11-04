use bevy::prelude::*;
use crate::prelude::*;
use bevy::ecs::batching::BatchingStrategy;
use rand::prelude::*;

use crate::galaxy::fleet::FleetColonyCrew;

use super::navigation::{
    NavPosition,
    Plan,
    Navigator,
    Action
};

pub fn nav_update_task_system(
    mut nav_query : Query<(&mut Navigator, &mut FleetColonyCrew)>,
    planet_query : Query<&Planet>,
) {
    for (mut nav,crew) in nav_query.iter_mut() {
        let Action::Idle = nav.action else { return; };
        if nav.plan_queue.len() > 0 { return; }

        if let Some(planet_id) = crew.destination {
            let planet = planet_query.get(planet_id).unwrap();
            nav.plan_queue.push(Plan::Colonise(planet_id));
            nav.plan_queue.push(Plan::ReachSystem(planet.star_id));
        }
    }
}

pub fn nav_find_colony_target_system(
    mut nav_query : Query<(&mut NavPosition, &mut FleetColonyCrew)>,
    system_query : Query<(&Star,&StarClaim)>,
    planet_query : Query<(&Planet,Entity), Without<Colony>>,
    hypernet : Res<Hypernet>,
){

    nav_query
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::fixed(32))
        .for_each(|(nav_pos,mut colony_fleet)|
    {
        if colony_fleet.destination.is_some() { return; };

        let mut rng = rand::thread_rng();

        let mut best_option : Option<(Entity)> = None;
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
                    best_option = Some(planet_entity);
                }
            }
        }

        if let Some(planet_entity) = best_option {
            colony_fleet.destination = Some(planet_entity);
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
    mut fleet_query : Query<(&Fleet, &mut FleetColonyCrew, &mut Navigator)>,
    mut star_query: Query<&mut StarClaim, With<Star>>,
    mut ev_colonise : EventReader<ColonisePlanetEvent>,
    mut commands : Commands
) {
    for ev in ev_colonise.read() {
        let Ok((fleet, mut colony_crew, mut nav)) = fleet_query.get_mut(ev.colony_fleet) else { continue; };
        let Ok((_planet, parent,colony)) = planet_query.get_mut(ev.planet_entity) else { continue; };
        let Ok(mut star_claim) = star_query.get_mut(parent.get()) else { continue; };

        if let Some(owner) = star_claim.owner {
            if owner != fleet.owner {
                info!("colonisation failed: System is already claimed by someone else");
                // reset the fleet
                nav.action = Action::Idle;
                nav.plan_queue.clear();
                colony_crew.destination = None;
                continue;
            }
        } else {
            star_claim.owner = Some(fleet.owner);
        }

        commands.entity(ev.colony_fleet).despawn();

        if let Some(mut colony) = colony {
            colony.population.add(colony_crew.colonists);
        } else {
            // NOTE
            // Creating an entity here can lead to some awkward behaviour, sometimes
            // EG. what if 2 colony ships do this on the same tick?
            // It would be convenient and quite sensible to just have the Colony component on all planets at all times, and inactive for uncolonised planets
            // Then preventing weird shit is quite trivial
            // (And achieving deterministic execution only requires us to ensure the loop iterates in the right order)
            commands.entity(ev.planet_entity)
                .insert(Colony {
                    owner : fleet.owner,
                    population : Population::new(colony_crew.colonists),
                    economy : Economy::new()
                });
        }
    }
}