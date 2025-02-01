use bevy::prelude::*;
mod time;
mod orbits;
mod economy;
mod schedule;

mod mission;

pub mod fleet_behaviour;

pub mod data;

pub use time::SimTime;

pub use schedule::{SimTick,SimPreTick,SimPostTick,SimStart,BuildGalaxyGraphics};

#[derive(Clone,Copy)]
pub enum SimulationMode {
    Slow,
    Normal,
    Fast,
    Fastest
}

#[derive(Resource)]
pub struct SimulationSettings {
    pub paused : bool,
    pub mode : SimulationMode,
    current_tick : i64,
    time_since_tick : f32
}

impl SimulationSettings {
    pub fn set_speed(&mut self, speed : SimulationMode) {
        self.time_since_tick = 0.0;
        self.mode = speed;
    }

    pub fn increase_speed(&mut self) {
        self.set_speed(match self.mode {
            SimulationMode::Slow => SimulationMode::Normal,
            SimulationMode::Normal => SimulationMode::Fast,
            SimulationMode::Fast => SimulationMode::Fastest,
            SimulationMode::Fastest => SimulationMode::Fastest,
        })
    }
    pub fn decrease_speed(&mut self) {
        self.set_speed(match self.mode {
            SimulationMode::Slow => SimulationMode::Slow,
            SimulationMode::Normal => SimulationMode::Slow,
            SimulationMode::Fast => SimulationMode::Normal,
            SimulationMode::Fastest => SimulationMode::Fast,
        })
    }
    pub fn toggle_pause(&mut self) {
        self.time_since_tick = 0.0;
        self.paused = !self.paused;
    }

    pub fn get_tick_interval(&self) -> Option<f32> {
        if self.paused { return None; }

        match self.mode {
            SimulationMode::Slow => Some(1.0),
            SimulationMode::Normal => Some(0.3),
            SimulationMode::Fast => Some(0.1),
            SimulationMode::Fastest => Some(0.015)
        }
    }
}

pub struct SimulationPlugin;

fn simulation_start_system(world : &mut World) {
    world.run_schedule(SimStart);
    world.run_schedule(SimPostTick);
    world.run_schedule(BuildGalaxyGraphics);
}

fn simulation_tick_system(world : &mut World) {
    let delta_seconds = world.resource::<Time>().delta_secs();
    let mut sim_settings = world.resource_mut::<SimulationSettings>();

    if let Some(tick_interval) = sim_settings.get_tick_interval() {
        sim_settings.time_since_tick += delta_seconds;

        if sim_settings.time_since_tick > tick_interval {
            sim_settings.time_since_tick = (sim_settings.time_since_tick - tick_interval).min(0.0);
            //info!("tick! {}", sim_settings.current_tick);
            sim_settings.current_tick += 1;
            world.run_schedule(SimPreTick);
            world.run_schedule(SimTick);
            world.run_schedule(SimPostTick);
        }
    }
}

use fleet_behaviour::colonisation;

impl Plugin for SimulationPlugin {
    fn build(&self, app : &mut App) {

        app.insert_resource(SimTime::new())
            .insert_resource(SimulationSettings{ mode : SimulationMode::Normal, paused : true, time_since_tick : 0.0, current_tick : 0})
            .add_systems(PostStartup,simulation_start_system)
            .add_systems(Update,(simulation_tick_system,crate::galaxy::fleet::fleet_preview_gizmos))
            .add_plugins(schedule::SchedulePlugin)
            .add_plugins(mission::planet_launch_colony::PlanetAutoColonyMissionPlugin)
            .add_event::<colonisation::ColonisePlanetEvent>();
    }
}