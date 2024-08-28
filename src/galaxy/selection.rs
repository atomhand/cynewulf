use bevy::prelude::*;
use crate::camera::{CameraSettings, CameraMode};
use crate::prelude::*;

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

pub enum InterfaceIdentifier {
    Entity(Entity),
    CurrentSystem,
    CurrentSelected,
    CurrentSystemOrbiter(u32),
}

#[derive(Resource,Clone)]
pub struct Selection {
    pub hovered : Option<Entity>,
    pub selected : Option<Entity>,
    pub selected_system : Option<Entity>, // The system (or star) of the selected entity
    pub zoomed_system : Option<Entity>,
}

#[derive(Component)]
pub struct SelectionProxy {
    pub target : InterfaceIdentifier
}

use std::collections::HashSet;
use bevy_mod_picking::pointer::InputPress;
use bevy_mod_picking::focus::HoverMap;
fn redirect_from_proxy(target : Entity, 
    selection_proxy_query : &Query<&SelectionProxy>,
    star_query : &Query<&Star>,
    selection : &Selection
) -> Option<Entity> {
    if let Ok(proxy) = selection_proxy_query.get(target) {
        match proxy.target {
            InterfaceIdentifier::Entity(entity) => {
                return Some(entity)
            },
            InterfaceIdentifier::CurrentSystem => {
                return selection.selected_system
            },
            InterfaceIdentifier::CurrentSelected => {                    
                return selection.selected
            },
            InterfaceIdentifier::CurrentSystemOrbiter(i) => {
                if let Some(system) = selection.selected_system {
                    if let Ok(star) = star_query.get(system) {
                        if (i as usize) < star.orbiters.len() {
                            return Some(star.orbiters[i as usize]);
                        }
                    }
                }
            }
        }
    }       
    None
}

fn update_hovered(
    mut selection : ResMut<Selection>,
    star_query : Query<&Star>,
    galaxy_selectable_query : Query<&GalaxySelectable,>,
    system_selectable_query : Query<&SystemSelectable>,
    selection_proxy_query : Query<&SelectionProxy>,
    camera_settings : Res<CameraSettings>,
    hover_map : Res<HoverMap>
) {
    selection.hovered = None;
    for (_pointer_id, hovered_entity,_hit) in hover_map
        .iter()
        .flat_map(|(id,hashmap)| hashmap.iter().map(|data| (*id,*data.0,data.1.clone())))
    {
        if let Some(proxy) = redirect_from_proxy(hovered_entity, &selection_proxy_query, &star_query, &selection) {
            selection.hovered = Some(proxy);
        }

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

// see https://github.com/aevyrie/bevy_mod_picking/blob/main/crates/bevy_picking_selection/src/lib.rs
fn update_selection (
    mut selection : ResMut<Selection>,
    mut pointer_down: EventReader<Pointer<Down>>,
    mut presses: EventReader<InputPress>,
    galaxy_selectable_query : Query<&GalaxySelectable,>,
    system_selectable_query : Query<&SystemSelectable>,
    selection_proxy_query : Query<&SelectionProxy>,
    no_deselect : Query<&NoDeselect>,
    star_query : Query<&Star>,
    camera_settings : Res<CameraSettings>,
    frame_count : Res<bevy::core::FrameCount>
) {

    let prev_selection = selection.clone();
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
        info!("handing pointer down event! frame: {}", frame_count.0);
        let target = redirect_from_proxy(*target, &selection_proxy_query, &star_query, &prev_selection).unwrap_or(*target);
        pointer_down_list.insert(*pointer_id);
        if let Some(selected) = selection.selected {
            let target_can_deselect = no_deselect.get(target).is_err();
            if target_can_deselect && selected != target {
                selection.selected = None;

                if camera_settings.camera_mode == CameraMode::Galaxy {
                    if let Some(system) = selection.selected_system {
                        if let Ok(star) = star_query.get(system) {
                            if !star.orbiters.contains(&target) {
                                selection.selected_system = None;
                            }
                        }
                    }
                }
            }
        } else {
            selection.selected = None;
            if camera_settings.camera_mode == CameraMode::Galaxy {
                selection.selected_system = None;
            }
        }

        if let Ok(_selectable) = galaxy_selectable_query.get(target) {
            if Some(target) == prev_selection.selected {
                selection.zoomed_system = Some(target);
            }
            selection.selected = Some(target);
            selection.selected_system = Some(target);
            info!("selecting a star (system!)");
        }
        else if let Ok(_selectable) = system_selectable_query.get(target) {
            selection.selected = Some(target);
            selection.selected_system = prev_selection.selected_system;
            info!("selecting a planet");
        }
    }

    if let Some(press) = presses
        .read()
        .filter(|p| p.is_just_down(PointerButton::Primary))
        .next()
    {
        info!("handling press event! frame: {}", frame_count.0);
        let id = press.pointer_id;
        if !pointer_down_list.contains(&id) {
            info!("clicked nothing. dropping selection");
            selection.selected = None;
            if camera_settings.camera_mode == CameraMode::Galaxy {
                selection.selected_system = None;
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