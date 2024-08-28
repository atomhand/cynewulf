#![feature(isqrt)]

use bevy::prelude::*;
use bevy_mod_picking;
use bevy::window::{PresentMode, WindowTheme};
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
            DefaultPlugins.set(WindowPlugin{
                primary_window: Some(Window {
                    title: "Cynewulf".into(),
                    name: Some("bevy.app".into()),
                    //resolution: (1920.,1080.).into(),
                    present_mode: PresentMode::AutoNoVsync,
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    window_theme: Some(WindowTheme::Dark),
                    ..default()
                }),
                ..default()
            }),
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