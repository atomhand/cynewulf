use bevy::prelude::*;

mod hud;
mod interface_state;
mod user_input;

pub struct InterfacePlugin;

impl Plugin for InterfacePlugin {
    fn build(&self, app : &mut App ) {
        app.add_plugins((
            hud::HudPlugin,
            user_input::InputPlugin,
            interface_state::UiStatePlugin,
        ));
    }
}