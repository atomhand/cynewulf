use crate::prelude::*;
use bevy::ecs::batching::BatchingStrategy;
use bevy::prelude::*;
use rand::prelude::*;

use crate::galaxy::fleet::FleetColonyCrew;

use super::navigation::{Action, NavPosition, Navigator, Plan};

pub fn nav_update_task_system(
    mut nav_query: Query<(&mut Navigator, &FleetColonyCrew)>,
    planet_query: Query<&Planet>,
) {
    for (mut nav, crew) in nav_query.iter_mut() {
        let Action::Idle = nav.action else {
            continue;
        };
        if !nav.plan_queue.is_empty() {
            continue;
        }

        if let Some(planet_id) = crew.destination {
            let planet = planet_query.get(planet_id).unwrap();
            nav.plan_queue.push(Plan::Colonise(planet_id));
            nav.plan_queue.push(Plan::ReachSystem(planet.star_id));
        }
    }
}

pub fn nav_find_colony_target_system(
    mut nav_query: Query<(&NavPosition, &Fleet, &mut FleetColonyCrew)>,
    system_query: Query<(&Star, &StarClaim)>,
    planet_query: Query<(&Planet, Entity, Option<&Colony>)>,
    nav_masks: Query<&NavigationMask>,
    hypernet: Res<Hypernet>,
) {
    nav_query
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::fixed(32))
        .for_each(|(nav_pos, fleet, mut colony_fleet)| {
            if let Some(dest) = colony_fleet.destination {
                // don't do validation calculations needlessly often
                if fleet.time_since_last_jump % 40 == 0 {
                    let empire = fleet.owner;
                    let nav_mask = nav_masks
                        .get(empire)
                        .expect("Nav find colony target: Can't find empire nav mask");
                    let nav_filter = nav_mask.to_filter(&hypernet);

                    let (dest_planet, _entity, _colony) = planet_query.get(dest).unwrap();

                    let path = nav_filter.find_path(nav_pos.root_system, dest_planet.star_id);

                    if path.is_none() {
                        colony_fleet.destination = None;
                    } else {
                        return;
                    }
                } else {
                    return;
                }
            };

            let mut rng = rand::rng();

            let mut best_option: Option<Entity> = None;
            let mut best_dist = i32::MAX;

            let empire = fleet.owner;

            let nav_mask = nav_masks
                .get(empire)
                .expect("Nav find colony target: Can't find empire nav mask");

            let nav_filter = nav_mask.to_filter(&hypernet);

            let dijkstra = nav_filter.dijkstra(&vec![nav_pos.root_system]);

            for (star_id, d_o) in dijkstra.iter().enumerate() {
                let Some(d) = d_o else {
                    continue;
                };

                let Ok((star, starclaim)) = system_query.get(hypernet.star(star_id as u32).entity)
                else {
                    continue;
                };

                if starclaim.owner.is_some() && starclaim.owner != Some(empire) {
                    continue;
                }

                for (_planet, planet_entity, colony) in star
                    .orbiters
                    .iter()
                    .filter_map(|planet_entity| planet_query.get(*planet_entity).ok())
                {
                    let weight = if let Some(_colony) = colony {
                        10000000 + rng.random_range(0..1000000)
                    } else {
                        d + rng.random_range(0..d + 1)
                    };

                    if weight < best_dist {
                        best_dist = weight;
                        best_option = Some(planet_entity);
                    }
                }
            }

            if let Some(planet_entity) = best_option {
                colony_fleet.destination = Some(planet_entity);
            };
        });
}

#[derive(Event)]
pub struct ColonisePlanetEvent {
    pub planet_entity: Entity,
    pub colony_fleet: Entity,
}

pub fn process_colonise_events(
    mut planet_query: Query<(&Planet, &Parent, Option<&mut Colony>)>,
    mut fleet_query: Query<(&Fleet, &mut FleetColonyCrew, &mut Navigator)>,
    mut star_query: Query<&mut StarClaim, With<Star>>,
    mut ev_colonise: EventReader<ColonisePlanetEvent>,
    sim_settings: Res<SimulationSettings>,
    mut commands: Commands,
) {
    for ev in ev_colonise.read() {
        let Ok((fleet, mut colony_crew, mut nav)) = fleet_query.get_mut(ev.colony_fleet) else {
            continue;
        };
        let Ok((_planet, parent, colony)) = planet_query.get_mut(ev.planet_entity) else {
            continue;
        };
        let Ok(mut star_claim) = star_query.get_mut(parent.get()) else {
            continue;
        };

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
            star_claim.claimed_tick = sim_settings.current_tick;
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
            commands.entity(ev.planet_entity).insert(Colony {
                claimed_tick: sim_settings.current_tick,
                owner: fleet.owner,
                population: Population::new(colony_crew.colonists),
                economy: Economy::new(),
            });
        }
    }
}
