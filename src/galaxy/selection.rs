use bevy::prelude::*;
use crate::camera_control::{CameraSettings, CameraMode};
use super::Star;


pub struct SelectionPlugin;

use bevy_mod_picking::prelude::*;
use bevy_mod_picking::backend::prelude::*;

impl Plugin for SelectionPlugin {
    fn build(&self, app : &mut App) {
        app.insert_resource(Selection{
            hovered : None,
            selected : None,
            selected_system : None,
            zoomed_system : None,
        })
            .add_systems(Update,selection_gizmos)
            .add_systems(PreUpdate,(update_selection,update_hovered).chain().in_set(PickSet::PostFocus));
    }
}

#[derive(Resource)]
pub struct Selection {
    pub hovered : Option<Entity>,
    pub selected : Option<Entity>,
    pub selected_system : Option<Entity>, // The system (or star) of the selected entity
    pub zoomed_system : Option<Entity>,
}

use std::collections::HashSet;
use bevy_mod_picking::pointer::InputPress;
use bevy_mod_picking::focus::HoverMap;

fn update_hovered(
    mut selection : ResMut<Selection>,
    galaxy_selectable_query : Query<&GalaxySelectable,>,
    system_selectable_query : Query<&SystemSelectable>,
    camera_settings : Res<CameraSettings>,
    hover_map : Res<HoverMap>
) {
    selection.hovered = None;
    for (_pointer_id, hovered_entity,_hit) in hover_map
        .iter()
        .flat_map(|(id,hashmap)| hashmap.iter().map(|data| (*id,*data.0,data.1.clone())))
    {
        match camera_settings.camera_mode {
            CameraMode::Galaxy => {
                if let Ok(_selectable) = galaxy_selectable_query.get(hovered_entity) {
                    selection.hovered = Some(hovered_entity);
                }
            },
            CameraMode::Star => {
                if let Ok(_selectable) = system_selectable_query.get(hovered_entity) {                    
                    selection.hovered = Some(hovered_entity);
                }
            }
        }
    }
}

fn update_selection (
    mut selection : ResMut<Selection>,
    mut pointer_down: EventReader<Pointer<Down>>,
    mut presses: EventReader<InputPress>,
    mut pointer_click: EventReader<Pointer<Click>>,
    galaxy_selectable_query : Query<&GalaxySelectable,>,
    system_selectable_query : Query<&SystemSelectable>,
    camera_settings : Res<CameraSettings>,
) {
    let mut pointer_down_list = HashSet::new();


    for Pointer {
        pointer_id,
        pointer_location : _,
        target,
        event: _,
     } in pointer_down
        .read()
        .filter(|pointer| pointer.event.button == PointerButton::Primary)
    {
        pointer_down_list.insert(*pointer_id);
        if let Some(selected) = selection.selected {
            let target_can_deselect = true;
            if target_can_deselect && selected != *target {
                selection.selected = None;
                if camera_settings.camera_mode == CameraMode::Galaxy {
                    selection.selected_system = None;
                }
            }
        } else {
            selection.selected = None;
            if camera_settings.camera_mode == CameraMode::Galaxy {
                selection.selected_system = None;
            }
        }
    }

    // If click nothing, deselect everything
    if let Some(press) = presses
        .read()
        .filter(|p| p.is_just_down(PointerButton::Primary))
        .next()
    {
        let id = press.pointer_id;
        if !pointer_down_list.contains(&id) {
            selection.selected = None;
            if camera_settings.camera_mode == CameraMode::Galaxy {
                selection.selected_system = None;
            }
        }
    }

    for Pointer {
        pointer_id  : _,
        pointer_location : _,
        target,
        event: _,
    } in pointer_click
        .read()
        .filter(|pointer| pointer.event.button == PointerButton::Primary)
    {
        match camera_settings.camera_mode {
            CameraMode::Galaxy => {
                if let Ok(_selectable) = galaxy_selectable_query.get(*target) {
                    if Some(*target) == selection.selected_system {
                        selection.zoomed_system = Some(*target);
                    }
                    selection.selected = Some(*target);
                    selection.selected_system = Some(*target);
                }
            },
            CameraMode::Star => {
                if let Ok(_selectable) = system_selectable_query.get(*target) {
                    selection.selected = Some(*target);
                }
            }
        }
    }
}

#[derive(Component)]
pub struct GalaxySelectable {
    pub radius : f32,
}

#[derive(Component)]
pub struct SystemSelectable {
    pub radius : f32,
}

fn selection_gizmos(
    selection : Res<Selection>,
    galaxy_selectable_query : Query<(&GalaxySelectable,&GlobalTransform)>,
    system_selectable_query : Query<(&SystemSelectable,&GlobalTransform)>,
    camera_settings : Res<CameraSettings>,
    mut gizmos : Gizmos
) {

    match camera_settings.camera_mode {
        CameraMode::Star => {
            if let Some(selected) = selection.selected {
                if let Ok((selectable,transform)) = system_selectable_query.get(selected) {
                    gizmos.circle(transform.translation(), Dir3::Y, selectable.radius * 0.9, Color::srgb(1.,0.4,0.));
                }
            }
            if let Some(hovered) = selection.hovered {
                if let Ok((selectable,transform)) = system_selectable_query.get(hovered) {
                    gizmos.circle(transform.translation(), Dir3::Y, selectable.radius, Color::WHITE);
                }
            }
        },
        CameraMode::Galaxy => {
            if let Some(selected) = selection.selected_system {
                if let Ok((selectable,transform)) = galaxy_selectable_query.get(selected) {
                    gizmos.circle(transform.translation(), Dir3::Y, selectable.radius * 0.9, Color::srgb(1.,0.4,0.));
                }
            }
            if let Some(hovered) = selection.hovered {
                if let Ok((selectable,transform)) = galaxy_selectable_query.get(hovered) {
                    gizmos.circle(transform.translation(), Dir3::Y, selectable.radius, Color::WHITE);
                }
            }
        }
    }
}

// deprecated
pub fn get_hovered(
    galaxy_selectable_query : Query<(Entity,&GalaxySelectable,&GlobalTransform)>,
    system_selectable_query : Query<(&SystemSelectable,&GlobalTransform)>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    stars : Query<&Children,With<Star>>,
    windows: Query<&Window>,
    camera_settings : Res<CameraSettings>,
    mut selection : ResMut<Selection>,
    mouse_button_input : Res<ButtonInput<MouseButton>>,
) {
    let (camera, camera_transform) = camera_query.single();

    let leftclick = mouse_button_input.just_pressed(MouseButton::Left);

    let old_selected_system = selection.selected_system;
    if leftclick {
        selection.selected = None;
        selection.selected_system = None;
    }

    if let Some(cursor_position) = windows.single().cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) {
            if let Some(distance) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) {
                let mouse_point = ray.get_point(distance);

                let mut n_dist = 10.0 * 10.0;
                let mut nearest : Option<Entity> = None;

                // iterate
                match camera_settings.camera_mode {
                    CameraMode::Star => {
                        if let Some(star) = camera_settings.star {
                            if let Ok(children) = stars.get(star) {
                                let mut system_entities : Vec<&Entity> = children.iter().collect();
                                system_entities.push(&star);
                                for entity in system_entities {
                                    if let Ok((selectable,transform)) = system_selectable_query.get(*entity) {
                                        let d = transform.translation().distance_squared(mouse_point);
                                        if d < n_dist && d < selectable.radius * selectable.radius {
                                            n_dist = d;
                                            nearest = Some(*entity);
                                        }
                                    }
                                }
                            }
                            if leftclick {
                                selection.selected = nearest;
                                selection.selected_system = Some(star);
                            }
                        }
                    },
                    CameraMode::Galaxy => {
                        for (entity,selectable,transform) in galaxy_selectable_query.iter() {
                            let d = transform.translation().distance_squared(mouse_point);
                            if d < n_dist && d < selectable.radius * selectable.radius {
                                n_dist = d;
                                nearest = Some(entity);
                            }
                        }
                        if leftclick {
                            selection.selected = nearest;
                            selection.selected_system = nearest;

                            if let Some(nearest) = nearest {
                                if let Some(old_selected_system) = old_selected_system {
                                    if old_selected_system == nearest {
                                        selection.zoomed_system = Some(nearest);
                                    }
                                }
                            }
                        }
                    }
                }

                selection.hovered = nearest;
            }
        }
    }

}