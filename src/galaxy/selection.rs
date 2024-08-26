use bevy::prelude::*;
use crate::camera::{CameraSettings, CameraMode};

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