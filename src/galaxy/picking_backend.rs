use bevy::prelude::*;

use bevy::picking::backend::prelude::*;
use bevy::render::view::RenderLayers;

use super::{selection::*, Star};
use crate::camera::{CameraMain, CameraMode, CameraSettings};
use crate::prelude::*;

pub struct PickingBackendPlugin;
impl Plugin for PickingBackendPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_hits.in_set(PickSet::Backend));
    }
}
use crate::simulation::fleet_behaviour::navigation::NavPosition;

fn update_hits(
    ray_map: Res<RayMap>,
    picking_cameras: Query<(&Camera, Option<&RenderLayers>)>,
    camera_settings: Res<CameraSettings>,
    galaxy_selectable_query: Query<(Entity, &GalaxySelectable, &GlobalTransform)>,
    system_selectable_query: Query<(&SystemSelectable, &GlobalTransform)>,
    fleet_query: Query<(Entity, &SystemSelectable, &NavPosition)>,
    camera: Query<&CameraMain>,
    stars: Query<&Children, With<Star>>,
    hypernet: Res<Hypernet>,
    mut output_events: EventWriter<PointerHits>,
) {
    let cam = camera.get_single().unwrap();
    let cam_transition = cam.adjusted_mode_transition();
    for (&ray_id, &ray) in ray_map.map().iter() {
        let Ok((camera, _cam_layers)) = picking_cameras.get(ray_id.camera) else {
            continue;
        };

        if let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) {
            let mouse_point = ray.get_point(distance);

            let mut n_dist = 100.0 * 100.0;
            let mut nearest: Option<Entity> = None;

            // iterate
            match camera_settings.camera_mode {
                CameraMode::Star => {
                    if let Some(star) = camera_settings.star {
                        if let Ok(children) = stars.get(star) {
                            let mut system_entities: Vec<&Entity> = children.iter().collect();
                            system_entities.push(&star);
                            for entity in system_entities {
                                if let Ok((selectable, transform)) =
                                    system_selectable_query.get(*entity)
                                {
                                    let d = transform.translation().distance_squared(mouse_point);
                                    if d < n_dist && d < selectable.radius * selectable.radius {
                                        n_dist = d;
                                        nearest = Some(*entity);
                                    }
                                }
                            }
                            // Fleets
                            for (entity, selectable, nav_pos) in &fleet_query {
                                let galaxy_pos = nav_pos.galaxy_view_translation(&hypernet);
                                let system_pos = nav_pos.system_view_translation(&hypernet);
                                let p = galaxy_pos.lerp(system_pos, cam_transition);
                                let d = p.distance_squared(mouse_point);
                                if d < n_dist && d < selectable.radius * selectable.radius {
                                    n_dist = d;
                                    nearest = Some(entity);
                                }
                            }
                        }
                    }
                }
                CameraMode::Galaxy => {
                    for (entity, selectable, transform) in galaxy_selectable_query.iter() {
                        let d = transform.translation().distance_squared(mouse_point);
                        if d < n_dist && d < selectable.radius * selectable.radius {
                            n_dist = d;
                            nearest = Some(entity);
                        }
                    }
                }
            }

            if let Some(nearest) = nearest {
                output_events.send(PointerHits::new(
                    ray_id.pointer,
                    vec![(nearest, HitData::new(ray_id.camera, distance, None, None))],
                    camera.order as f32,
                ));
            }
        }
    }
}
