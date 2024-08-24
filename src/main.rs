#![feature(isqrt)]

use bevy::prelude::*;
use bevy_mod_picking;

mod camera;
mod galaxy;
mod graphics;
mod simulation;
mod interface;
mod markov_chain;
mod prelude;
mod util;

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
            bevy_mod_picking::DefaultPickingPlugins
        ))
        .run();
}

/*
fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a world position based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Y)) else {
        return;
    };

    let point = ray.get_point(distance);

    //gizmos.circle(point, Direction3d::Y, 10., Color::WHITE);
}
*/
