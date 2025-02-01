use bevy::prelude::*;
use crate::camera::{CameraSettings, CameraMode};
use crate::prelude::*;

pub struct SelectionPlugin;
use std::collections::HashSet;

use bevy::picking::{
    prelude::*,
    backend::prelude::*,
    focus::HoverMap
};

impl Plugin for SelectionPlugin {
    fn build(&self, app : &mut App) {
        app.insert_resource(Selection{
            hovered : None,
            selected : None,
            hovered_empire : None,
            selected_empire : None,
            selected_system : None,
            zoomed_system : None,
        })
            .add_systems(Update,selection_gizmos)
            .add_systems(PreUpdate,(resolve_proxies,update_selection,update_hovered,update_selected_empire).chain().in_set(PickSet::PostFocus));
    }
}

pub enum InterfaceIdentifier {
    // Entity(Entity),
    PlayerEmpire,
    // CurrentSystem,
    // CurrentSelected,
    CurrentSystemOrbiter(u32),
    CurrentSelectedFleet(u32),
    EmpirePlanet(u32),
    EmpireStar(u32)
}

pub enum SelectionState {
    None,
    SemiHovered,
    Hovered,
    Selected,
    HoverSelected
}
impl SelectionState {
    pub fn as_colour(&self) -> Color {
        match *self {
            SelectionState::None => Color::NONE,
            SelectionState::Hovered => Color::linear_rgba(1.0,165./255.,0., 1.0),
            SelectionState::Selected => Color::WHITE,
            SelectionState::SemiHovered => Color::linear_rgb(0.3,0.3,0.3),
            SelectionState::HoverSelected => Color::linear_rgba(1.0,80./255.,0., 1.0),
        }
    }
    // this is for UI or whatever where you want unselected borders to be gray rather than blank
    pub fn as_colour_with_default(&self, def : Color) -> Color {
        match *self {
            SelectionState::None => def,
            SelectionState::Hovered => Color::linear_rgba(1.0,165./255.,0., 1.0),
            SelectionState::Selected => Color::WHITE,
            SelectionState::SemiHovered => Color::linear_rgb(0.3,0.3,0.3),
            SelectionState::HoverSelected => Color::linear_rgba(1.0,80./255.,0., 1.0),
        }
    }
}

#[derive(Resource,Clone)]
pub struct Selection {
    pub hovered : Option<Entity>,
    pub selected : Option<Entity>,
    pub hovered_empire : Option<Entity>,
    pub selected_empire : Option<Entity>,
    pub selected_system : Option<Entity>, // The system (or star) of the selected entity
    pub zoomed_system : Option<Entity>,
}

impl Selection {
    pub fn get_selection_state(&self, entity : Entity) -> SelectionState {
        let hovered = Some(entity) == self.hovered;
        let selected = Some(entity) == self.selected;

        let empire_hovered = Some(entity) == self.hovered_empire;
        let empire_selected = Some(entity) == self.selected_empire;

        if selected && hovered {
            SelectionState::HoverSelected
        } else if selected {
            SelectionState::Selected
        } else if hovered {
            SelectionState::Hovered
        } else if empire_hovered || empire_selected {
            SelectionState::SemiHovered
        } else {
            SelectionState::None
        }
    }
}

#[derive(Component)]
pub struct SelectionProxy {
    pub target : InterfaceIdentifier,
    pub resolved_target : Option<Entity>
}

impl SelectionProxy {
    pub fn new(target : InterfaceIdentifier) -> Self {
        Self {
            target,
            resolved_target : None
        }
    }
}

fn resolve_proxies(
    mut proxies : Query<&mut SelectionProxy>,
    star_query : Query<&Star,Without<SelectionProxy>>,
    fleet_query : Query<&Fleet>,
    selection : Res<Selection>,
    player_empire : Res<crate::galaxy::empire::PlayerEmpire>,
    empire_query : Query<(&Empire,&EmpireIndex)>
) {
    for mut proxy in proxies.iter_mut() {
        proxy.resolved_target = match proxy.target {
            /*
            InterfaceIdentifier::Entity(entity) => {
                Some(entity)
            },
            InterfaceIdentifier::CurrentSystem => {
                selection.selected_system
            },
            InterfaceIdentifier::CurrentSelected => {                    
                selection.selected
            },
            */
            InterfaceIdentifier::PlayerEmpire => {
                player_empire.empire
            },
            InterfaceIdentifier::CurrentSystemOrbiter(i) => {
                selection.selected_system
                    .and_then(|star_ent| star_query.get(star_ent).ok())
                    .and_then(|star| if (i as usize) < star.orbiters.len() {
                        Some(star.orbiters[i as usize])
                    } else {
                        None
                    })
            },
            InterfaceIdentifier::CurrentSelectedFleet(i) => {
                if let Some(selected) = selection.selected {
                    if i == 0
                    && fleet_query.contains(selected) {
                            selection.selected
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            InterfaceIdentifier::EmpireStar(i) => {
                if let Some(empire_entity) = player_empire.empire {                    
                    let (_empire,index) = empire_query.get(empire_entity).unwrap();
                    if (i as usize) < index.systems.len() {
                        Some(index.systems[i as usize])
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            InterfaceIdentifier::EmpirePlanet(i) => {
                if let Some(empire_entity) = player_empire.empire {                    
                    let (_empire,index) = empire_query.get(empire_entity).unwrap();
                    if (i as usize) < index.colonies.len() {
                        Some(index.colonies[i as usize])
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };
    }
}


fn update_hovered(
    mut selection : ResMut<Selection>,
    galaxy_selectable_query : Query<(&GalaxySelectable,&StarClaim)>,
    system_selectable_query : Query<&SystemSelectable>,
    selection_proxy_query : Query<&SelectionProxy>,
    mouse_buttons : Res<ButtonInput<MouseButton>>,
    camera_settings : Res<CameraSettings>,
    hover_map : Res<HoverMap>
) {
    selection.hovered = None;
    for (_pointer_id, hovered_entity,_hit) in hover_map
        .iter()
        .flat_map(|(id,hashmap)| hashmap.iter().map(|data| (*id,*data.0,data.1.clone())))
    {
        if let Some(proxy) = selection_proxy_query.get(hovered_entity).ok().and_then(|x| x.resolved_target) {
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

    if mouse_buttons.just_pressed(MouseButton::Right) {
        if camera_settings.camera_mode == CameraMode::Galaxy 
        && selection.hovered == selection.selected {
            let Some((_,claim)) = selection.selected.map(|selected| galaxy_selectable_query.get(selected).ok()).flatten() else { return; };
            
            if let Some(empire) = claim.owner {
                selection.selected = Some(empire);
            }
        }
    }
}

pub fn update_selected_empire (
    mut selection : ResMut<Selection>,
    empire_query : Query<&Empire>,
    star_query : Query<&StarClaim>
) {
    if let Some(selected) = selection.selected {
        if empire_query.contains(selected) {
            selection.selected_empire = Some(selected);
        }
        if let Ok(starclaim) = star_query.get(selected) {
            selection.selected_empire = starclaim.owner;
        }
    } else {
        selection.selected_empire = None;
    }
    
    if let Some(hovered) = selection.hovered {
        if empire_query.contains(hovered) {
            selection.hovered_empire = Some(hovered);
        }
        if let Ok(starclaim) = star_query.get(hovered) {
            selection.hovered_empire = starclaim.owner;
        }
    } else {
        selection.hovered_empire = None;
    }
}

// see https://github.com/aevyrie/bevy_mod_picking/blob/main/crates/bevy_picking_selection/src/lib.rs
fn update_selection (
    mut selection : ResMut<Selection>,
    mut pointer_down: EventReader<Pointer<Down>>,
    galaxy_selectable_query : Query<&GalaxySelectable,>,
    system_selectable_query : Query<&SystemSelectable>,
    selection_proxy_query : Query<&SelectionProxy>,
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
        //let target = redirect_from_proxy(*target, &selection_proxy_query, &star_query, &prev_selection).unwrap_or(*target);
        let target = selection_proxy_query.get(*target).ok().and_then(|x| x.resolved_target).unwrap_or(*target);

        pointer_down_list.insert(*pointer_id);
        if let Some(selected) = selection.selected {
            let target_can_deselect = true;//no_deselect.get(target).is_err();
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
        else {
            selection.selected = Some(target);
            info!("selecting something else...");
        }
    }

    /*
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
    */
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
    //galaxy_selectable_query : Query<(&GalaxySelectable,&GlobalTransform)>,
    system_selectable_query : Query<(&SystemSelectable,&GlobalTransform)>,
    camera_settings : Res<CameraSettings>,
    mut gizmos : Gizmos
) {

    match camera_settings.camera_mode {
        CameraMode::Star => {
            if let Some(selected) = selection.selected {
                if let Ok((selectable,transform)) = system_selectable_query.get(selected) {
                    gizmos.circle(Isometry3d::from_translation(transform.translation()), selectable.radius * 0.9, Color::srgb(1.,0.4,0.));
                }
            }
            if let Some(hovered) = selection.hovered {
                if let Ok((selectable,transform)) = system_selectable_query.get(hovered) {
                    gizmos.circle(Isometry3d::from_translation(transform.translation()), selectable.radius, Color::WHITE);
                }
            }
        },
        CameraMode::Galaxy => {
            /* 
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
            */
        }
    }
}