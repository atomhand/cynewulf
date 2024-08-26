#![feature(isqrt)]

use bevy::prelude::*;
use bevy_mod_picking;

mod camera;
mod galaxy;
mod graphics;
mod simulation;
mod interface;
mod prelude;
mod util;
mod generators;

use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    App::new()
        .add_plugins((
            DefaultPlugins,
            galaxy::GalaxySetupPlugin,
            graphics::GraphicsPlugin,
            simulation::SimulationPlugin,
            interface::InterfacePlugin,
            camera::CameraPlugin,
            bevy_mod_picking::DefaultPickingPlugins,
            generators::GalaxyGenerationPlugin
        ))
        .run();
}