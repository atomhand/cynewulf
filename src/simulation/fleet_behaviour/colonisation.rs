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

use crate::galaxy::navigation_filter::{NavigationFilter,NavigationMask};

pub fn nav_find_colony_target_system(
    mut nav_query : Query<(&mut NavPosition, &Fleet, &mut FleetColonyCrew)>,
    system_query : Query<(&Star,&StarClaim)>,
    planet_query : Query<(&Planet,Entity), Without<Colony>>,
    nav_masks : Query<&NavigationMask>,
    hypernet : Res<Hypernet>,
){

    nav_query
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::fixed(32))
        .for_each(|(nav_pos, fleet, mut colony_fleet)|
    {
        if colony_fleet.destination.is_some() { return; };

        let mut rng = rand::thread_rng();

        let mut best_option : Option<(Entity)> = None;
        let mut best_dist = i32::MAX;

        let empire = fleet.owner;

        let nav_mask = nav_masks.get(empire).expect("Nav find colony target: Can't find empire nav mask");
    
        let nav_filter = nav_mask.to_filter(&hypernet);

        let dijkstra = nav_filter.dijkstra(&vec![nav_pos.root_system]);

        for star_id in 0..dijkstra.len() {
            let Some(d) = dijkstra[star_id] else { continue; };

            let Ok((star,starclaim)) = system_query.get(hypernet.node(star_id as u32).star) else { continue; };

            if starclaim.owner != None && starclaim.owner != Some(empire) { continue; }

            let Some((_planet,planet_entity)) = star.orbiters.iter().filter_map(|x| planet_query.get(*x).ok()).choose(&mut rng) else { continue; };

            if d < best_dist {
                best_dist = d;
                best_option = Some(planet_entity);
            }
        }

        /* 
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
        */

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