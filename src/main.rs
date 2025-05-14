use bevy::prelude::*;
use bevy::window::{PresentMode, WindowTheme};
mod camera;
mod galaxy;
mod graphics;
mod simulation;
//mod interface;
mod generators;
mod interface_new;
mod prelude;
mod util;

use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
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
            //bevy_framepace::FramepacePlugin,
            simulation::SimulationPlugin,
            galaxy::GalaxySetupPlugin,
            graphics::GraphicsPlugin,
            interface_new::InterfacePlugin,
            //interface::InterfacePlugin,
            camera::CameraPlugin,
            generators::GalaxyGenerationPlugin,
        ))
        .run();
}
