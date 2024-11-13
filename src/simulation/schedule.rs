use bevy::prelude::*;

use bevy::ecs::schedule::ScheduleLabel;

use super::fleet_behaviour::{navigation,colonisation};
use super::time;
use super::orbits;
use super::economy::demography_system;

pub struct SchedulePlugin;

impl Plugin for SchedulePlugin {
    fn build(&self, app : &mut App ) {
        let mut simulation_schedule = Schedule::new(SimTick);
        // This needs to be split into multiple schedules at some point
        simulation_schedule.add_systems((
            time::tick_date_system,
            orbits::update_orbiters,
            demography_system::update_population,
            (navigation::navigation_update_nav_system,
            colonisation::nav_find_colony_target_system,
            colonisation::nav_update_task_system,
            colonisation::process_colonise_events).chain()
            //crate::galaxy::indexes::empires_index::update_empire_index_system // this could be organised in a more hierarchical way
        ));

        let mut pre_tick_schedule = Schedule::new(SimPreTick);
        pre_tick_schedule.add_systems(
            crate::galaxy::navigation_filter::update_empire_navigation_masks
        );

        let post_tick_schedule = Schedule::new(SimPostTick);
        let sim_start_schedule = Schedule::new(SimStart);
        let build_graphics_schedule = Schedule::new(BuildGalaxyGraphics);

        app
            .add_schedule(simulation_schedule)
            .add_schedule(pre_tick_schedule)
            .add_schedule(sim_start_schedule)
            .add_schedule(post_tick_schedule)
            .add_schedule(build_graphics_schedule);
    }
}

#[derive(ScheduleLabel,Debug,Hash,PartialEq,Eq,Clone)]
pub struct SimStart;

#[derive(ScheduleLabel,Debug,Hash,PartialEq,Eq,Clone)]
pub struct SimPreTick;
#[derive(ScheduleLabel,Debug,Hash,PartialEq,Eq,Clone)]
pub struct SimTick;

#[derive(ScheduleLabel,Debug,Hash,PartialEq,Eq,Clone)]
pub struct BuildGalaxyGraphics;

#[derive(ScheduleLabel,Debug,Hash,PartialEq,Eq,Clone)]
pub struct SimPostTick;