use bevy::prelude::*;
pub struct InputPlugin;

use crate::simulation::SimulationSettings;

pub fn time_control_system(mut sim_settings : ResMut<SimulationSettings>, keys : Res<ButtonInput<KeyCode>>) {
    
    if keys.just_pressed(KeyCode::Space) {
        sim_settings.toggle_pause();
    }
    if keys.just_pressed(KeyCode::NumpadAdd) {
        sim_settings.increase_speed();
    }
    if keys.just_pressed(KeyCode::NumpadSubtract) {
        sim_settings.decrease_speed();
    }
}

impl Plugin for InputPlugin {
    fn build(&self, app : &mut App ) {
        app.add_systems(Update, time_control_system);
    }
}