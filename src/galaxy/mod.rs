use bevy::prelude::*;

mod picking_backend;

pub mod selection;
pub use selection::Selection;

pub mod galaxy_config;
pub use galaxy_config::GalaxyConfig;

mod pathfinding;
pub use pathfinding::Pathfinding;

pub mod galaxy_generation;

pub mod description;
pub use description::Description;

mod star;
pub use star::{Star,OverlaysTriangulationVertex};

mod hypernet;
pub use hypernet::Hypernet;

mod planet;
pub use planet::Planet;

mod empires_placement;
pub struct GalaxySetupPlugin;
/* 
#[derive(Resource)]
pub struct SelectedObject {
    pub hovered_star : Option<(Vec3,Entity)>,
}

fn update_hovered_star(
    stars : Query<(Entity,&Star)>, 
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Query<&Window>,
    mut selected_object : ResMut<SelectedObject>) {
    
    let (camera, camera_transform) = camera_query.single();
    if let Some(cursor_position) = windows.single().cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            if let Some(distance) = ray.intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Y)) {
                let mouse_point = ray.get_point(distance);

                let mut n_dist = 10.0 * 10.0;
                let mut nearest : Option<(Vec3,Entity)> = None;
                for (entity,star) in &stars {
                    let d = star.pos.distance_squared(mouse_point);
                    if d < n_dist {
                        n_dist = d;
                        nearest = Some((star.pos,entity));
                    }
                }
                selected_object.hovered_star = nearest;
            }
        }
    }
}
*/


impl Plugin for GalaxySetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((selection::SelectionPlugin, picking_backend::PickingBackendPlugin))
            .insert_resource(GalaxyConfig::default())
            .insert_resource(Hypernet::new())
            //.insert_resource(SelectedObject{hovered_star : None})
            .add_systems(Startup, (galaxy_generation::setup_stars,empires_placement::place_star_empires.after(galaxy_generation::setup_stars)))
            .add_systems(Update, description::update_descriptions_system);
    }
}