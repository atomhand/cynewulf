use bevy::prelude::*;
use crate::galaxy::Hypernet;
use crate::prelude::*;

#[derive(Component)]
pub struct NavPosition {
    pub root_system : u32,
    pub offset : NavOffset
}

#[derive(Clone)]
pub struct HyperlaneLocalPos {
    //star_a : u32, // edge id in the hypernet
    star_b : u32,
    progress : i32,
    distance : i32
}

#[derive(Clone)]
pub enum NavOffset {
    Star(Vec3),
    Hyperlane(HyperlaneLocalPos),
}

impl NavPosition {
    pub fn system_view_translation(&self, hypernet : &Hypernet) -> Vec3 {
        let star_pos = hypernet.graph.node_weight(self.root_system.into()).unwrap().pos;

        match self.offset {
            NavOffset::Star(offset) => {
                star_pos.as_dvec3() + offset.as_dvec3()
            },
            NavOffset::Hyperlane(HyperlaneLocalPos {
                star_b,
                progress,
                distance
            }) => {
                let star_b_pos = hypernet.graph.node_weight(star_b.into()).unwrap().pos;
                let dir = (star_b_pos.as_dvec3()-star_pos.as_dvec3()).normalize();

                let a = star_pos.as_dvec3() + dir * GalaxyConfig::HYPERLANE_VISUAL_STAR_CLEARANCE as f64;
                let b = star_b_pos.as_dvec3() - dir * GalaxyConfig::HYPERLANE_VISUAL_STAR_CLEARANCE as f64;
                a.lerp(b, (progress as f64 / distance as f64).min(1.0))
            }
        }.as_vec3()
    }
    pub fn galaxy_view_translation(&self, hypernet : &Hypernet) -> Vec3 {
        let star_pos = hypernet.graph.node_weight(self.root_system.into()).unwrap().pos;

        match self.offset {
            NavOffset::Star(offset) => {
                star_pos.as_dvec3() + offset.as_dvec3() * GalaxyConfig::HYPERLANE_VISUAL_STAR_CLEARANCE as f64 / (7.0 * GalaxyConfig::AU_SCALE as f64)
            },
            NavOffset::Hyperlane(HyperlaneLocalPos {
                star_b,
                progress,
                distance
            }) => {
                let star_b_pos = hypernet.graph.node_weight(star_b.into()).unwrap().pos;
                let dir = (star_b_pos.as_dvec3()-star_pos.as_dvec3()).normalize();
        
                let a = star_pos.as_dvec3() + dir * GalaxyConfig::HYPERLANE_VISUAL_STAR_CLEARANCE as f64;
                let b = star_b_pos.as_dvec3() - dir * GalaxyConfig::HYPERLANE_VISUAL_STAR_CLEARANCE as f64;
                a.lerp(b, (progress as f64 / distance as f64).min(1.0))
            }
        }.as_vec3()
    }
}

#[derive(Component)]
pub struct Navigator {
    pub action : Action,
    pub plan_queue : Vec<Plan>,
    pub speed : f32,
    pub hyperspeed : i32
}

pub enum Action {
    Move(Vec3),
    Jumping,
    Colonise((Entity,i32)), // Duration
    BeingDestroyed,
    Idle
}

#[derive(Clone,Copy)]
pub enum Plan {
    ReachSystem(u32),
    ReachPoint(Vec3),
    Jump(u32),
    Colonise(Entity) // Planet
}

// Finds the entry/exit point in the system of "star" for the hyperlane connecting to "other"
// Maybe it would be neater + better for this function to be a member of Hypernet
// In order do accomplish that cleanly, the necessary information (star system radius) should be stored in the hypernet node weights.
// -- Which would be fine, since it's almost certainly going to be a fixed value for each star.
fn hyperlane_transit_point(star : &Star, other : Vec3) -> Vec3 {
    let dir = (other.as_dvec3() - star.pos.as_dvec3()).normalize();
    (dir * star.system_radius_actual() as f64).as_vec3()
}
use crate::galaxy::Fleet;
#[derive(Event)]
pub struct ColonisePlanetEvent {
    planet_entity : Entity,
    colony_fleet : Entity
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

use rand::prelude::*;

pub fn nav_find_colony_target_system(
    mut nav_query : Query<(&mut NavPosition, &mut Navigator, &Fleet)>,
    system_query : Query<(&Star,&StarClaim),Without<Navigator>>,
    planet_query : Query<(&Planet,Entity), (Without<Colony>,Without<Navigator>)>,
    hypernet : Res<Hypernet>,
){
    let mut rng = rand::thread_rng();

    for(nav_pos,mut nav, _fleet) in nav_query.iter_mut() {
        let Action::Idle = nav.action else { continue; };
        if nav.plan_queue.len() > 0 { continue; }

        let mut best_option : Option<(u32,Entity)> = None;
        let mut best_dist = usize::MAX;
        for(star,starclaim) in system_query.iter() {
            if let Some(_star_owner) = starclaim.owner {
                continue;
                //if star_owner != fleet.owner { continue; }
            }
    
            let Some((_planet,planet_entity)) = star.orbiters.iter().filter_map(|x| planet_query.get(*x).ok()).choose(&mut rng) else { continue; };
    
            if let Some(path) = hypernet.find_path(nav_pos.root_system, star.node_id) {
                let d = path.len();
    
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
    }
}

pub fn navigation_update_nav_system(
    mut nav_query : Query<(&mut NavPosition, &mut Navigator, Entity)>,
    system_query : Query<(&Star,&StarClaim),Without<Navigator>>,
    planet_query : Query<&Planet, Without<Navigator>>,
    hypernet : Res<Hypernet>,
    mut ev_colonise : EventWriter<ColonisePlanetEvent>
) {
    // 1. If we are in a hyperlane, update travel progress
    // -- If we finished travelling, set location to the new system and mark that we're idle
    // 2. If there is an active Action,
    // -- check whether it is completed, or invalid. (In Which case trigger completion actions and potentially mark Idle - or eg. Jumping if the action automatically triggers a new one)
    // -- Execute it (eg. if it's a movement action, move...)
    // 3. If there is not an active Action, try to grab the next action. (This could come from the action queue, or from an automation policy)
    // -- When an action is grabbed from the queue, the behaviour depends on the action
    // - An action like Jump is consumed and attempts to execute immediately
    // - An action like Move is consumed and creates an analogue action on the execution slot
    // - An action like "Reach Destination" pushes new Move&Jump actions to the front of the queue, and isn't actually removed (unless the destination has been reached..)

    for(mut nav_pos, mut nav, fleet_entity) in nav_query.iter_mut() {
        // 1. If we are in a hyperlane, update travel progress
        // -- If we finished travelling, set location to the new system and mark that we're idle
        if let NavOffset::Hyperlane(HyperlaneLocalPos {
            //star_a,
            star_b,
            progress,
            distance
        }) = nav_pos.offset {
            let star_a_node = hypernet.graph.node_weight(nav_pos.root_system.into()).unwrap();
            let (star_a,_) = system_query.get(star_a_node.star).unwrap();
            let star_b_node = hypernet.graph.node_weight(star_b.into()).unwrap();
            let (star_b_ref,_) = system_query.get(star_b_node.star).unwrap();

            let progress =progress + nav.hyperspeed;

            *nav_pos = if progress >= distance { // Finished Jumping
                let hyperspace_exit_point = hyperlane_transit_point(star_b_ref, star_a.pos);
                NavPosition {
                    root_system : star_b,
                    offset : NavOffset::Star(hyperspace_exit_point)
                }
            } else {
                NavPosition {
                    root_system : nav_pos.root_system,
                    offset : NavOffset::Hyperlane(HyperlaneLocalPos {
                        //star_a,
                        star_b,
                        progress,
                        distance
                    })
                }
            };
        }
        // 2. If there is an active Action,
        // -- check whether it is completed, or invalid. (In Which case trigger completion actions and potentially mark Idle - or eg. Jumping if the action automatically triggers a new one)
        // -- Execute it (eg. if it's a movement action, move...)
        nav.action = match nav.action {
            Action::Idle => {
                Action::Idle
            },
            Action::Jumping => {
                match nav_pos.offset {
                    NavOffset::Hyperlane(_) => Action::Jumping,
                    _ => Action::Idle,
                }
            },
            Action::BeingDestroyed => {
                // ... Should we check to see if a fleet gets stuck in this state?
                Action::BeingDestroyed
            },
            Action::Colonise((planet_entity, duration)) => {
                if let NavOffset::Star(offset) = &mut nav_pos.offset {
                    if let Ok(planet) = planet_query.get(planet_entity) {
                        // track position to the target planet..
                        let dest = planet.system_local_pos();
                        let dir = (dest - *offset).normalize_or_zero();
                        let dist = dest.distance(*offset);
                        let speed = nav.speed.min(dist);
                        *offset = *offset + dir * speed;

                        // I Guess the action could fail here if the ship can't keep up with the planet?
                        // Not something that should ever actually happen though.
                        let mut dur = duration;
                        // Colonisation progresses 
                        if dist <= speed {
                            dur = duration - 1;                            
                        } 
                        if dur <= 0 {
                            // SEND COLONISATION EVENT
                            info!("Colonising!");
                            ev_colonise.send(ColonisePlanetEvent {
                                planet_entity,
                                colony_fleet : fleet_entity
                            });
                            Action::BeingDestroyed
                        } else {
                            Action::Colonise((planet_entity,dur))
                        }
                    } else {
                        Action::Idle
                    }
                } else {
                    Action::Idle
                }
            },               
            Action::Move(dest) => {
                if let NavOffset::Star(offset) = &mut nav_pos.offset {
                    let dir = (dest - *offset).normalize_or_zero();
                    let dist = dest.distance(*offset);
                    let speed = nav.speed.min(dist);
                    *offset = *offset + dir * speed;
                    
                    if dist <= speed { // reached destination, move is finished...
                        Action::Idle
                    } else { // did not reach destination..
                        Action::Move(dest)
                    }
                } else {
                    Action::Idle
                }
            }
        };

        // 3. If there is not an active Action, try to grab the next action. (This could come from the action queue, or from an automation policy)
        // -- When an action is grabbed from the queue, the behaviour depends on the action
        // - An action like Jump is consumed and attempts to execute immediately
        // - An action like Move is consumed and creates an analogue action on the execution slot
        // - An action like "Reach Destination" pushes new Move&Jump actions to the front of the queue, and isn't actually removed (unless the destination has been reached..)

        while let Action::Idle = nav.action {
            assert!(if let NavOffset::Star(_) = nav_pos.offset { true } else {false }, "Navigation: It is invalid for a fleet to be marked Idle while in hyperlane transit!");

            // FOR TESTING
            // If there's no plan, make one!
            if nav.plan_queue.is_empty() {
                break;

                //let mut rng = rand::thread_rng();
                //nav.plan_queue.push(Plan::ReachSystem(hypernet.graph.node_indices().choose(&mut rng).unwrap().index() as u32));
            }

            let top = nav.plan_queue[nav.plan_queue.len()-1];
            match top {
                Plan::Jump(dest_system_id) => {
                    let next_system_node = hypernet.graph.node_weight(dest_system_id.into()).unwrap();
                    let root_system_node = hypernet.graph.node_weight(nav_pos.root_system.into()).unwrap();
                    let (root_system_star,_) = system_query.get(root_system_node.star).unwrap();

                    let transit_point = hyperlane_transit_point(root_system_star, next_system_node.pos);

                    // An alternative (better) way to validate for robustness here would be for the Jump plan to store Origin as well as destination..
                    assert!(hypernet.graph.neighbors(nav_pos.root_system.into()).any(|x| x == dest_system_id.into()), "Navigation: Jump must target a neighbour in the hypernet!");

                    if let NavOffset::Star(offset) = nav_pos.offset {
                        // close enough to jump, let's go!
                        if transit_point.distance(offset) <= nav.speed {
                            nav.plan_queue.pop();
                            nav.action = Action::Jumping;

                            let edge = hypernet.graph.edge_weight(hypernet.graph.find_edge(nav_pos.root_system.into(),dest_system_id.into()).unwrap()).unwrap();
                            *nav_pos = NavPosition {
                                root_system : nav_pos.root_system,
                                offset : NavOffset::Hyperlane(HyperlaneLocalPos {
                                    star_b : dest_system_id,
                                    progress : 0,
                                    distance : edge.length
                                })
                            }
                        } else {
                            // not close enough, plan is invalid
                            info!("Navigation: Invalid Jump plan (Too far from transit point). Dropping navigation queue.");
                            nav.plan_queue.clear();
                        }
                    } else {
                        info!("Navigation: Invalid Jump plan (In Hyperspace). Dropping navigation queue.");
                        nav.plan_queue.clear();
                    }
                },
                Plan::Colonise(planet_entity) => {
                    // Check we are in the right system??

                    nav.plan_queue.pop();
                    nav.action = Action::Colonise((planet_entity,60));
                },
                Plan::ReachPoint(dest_point) =>  {
                    nav.plan_queue.pop();
                    nav.action = Action::Move(dest_point);
                },
                Plan::ReachSystem(dest_system_id) =>  {
                    if dest_system_id == nav_pos.root_system { // We're already there, so we can consider this plan finished...
                        nav.plan_queue.pop();
                    } else {
                        if let Some(path) = hypernet.find_path(nav_pos.root_system, dest_system_id) {
                            assert!(nav_pos.root_system == path[0], "path[0] doesn't match path origin!");

                            if path.len() > 1 {
                                let next_system = path[1];
                                let next_system_node = hypernet.graph.node_weight(next_system.into()).unwrap();
                                let root_system_node = hypernet.graph.node_weight(nav_pos.root_system.into()).unwrap();
                                let (root_system_star,_) = system_query.get(root_system_node.star).unwrap();
    
                                nav.plan_queue.push(Plan::Jump(next_system));
                                nav.plan_queue.push(Plan::ReachPoint(hyperlane_transit_point(root_system_star, next_system_node.pos)));
                            } else {
                                // already there..
                                nav.plan_queue.pop();
                            }

                        } else {
                            // No valid path, so cancel it

                            // .. This also needs to cancel all further queued plans under the current design
                            // It may be desirable to adjust things so that after a failed plan we gracefully moves along the queue
                            // .. or not. Kinda debatable situation
                            info!("Navigation: ReachSystem plan couldn't find a path. Dropping navigation queue.");
                            nav.plan_queue.clear();
                        }
                    }
                }
            }
        }
    }
}