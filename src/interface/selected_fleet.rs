use bevy::prelude::*;
use crate::galaxy::{Selection,GalaxyConfig};
use bevy_mod_picking::prelude::*;
use crate::prelude::*;
use super::UiConsts;
use crate::galaxy::selection::{SelectionProxy,InterfaceIdentifier};
use crate::galaxy::Description;

/// Marker to find the text entity so we can update it
#[derive(Component)]
struct SelectionPanel;

#[derive(Component)]
struct SelectionPanelTabRoot {
    slot : i32
}

#[derive(Component)]
struct SelectionPanelTabHeader {
    slot : i32
}
#[derive(Component)]
struct SelectionPanelTabDetails {
    slot : i32
}

pub struct FleetSelectionPanelPlugin;

impl Plugin for FleetSelectionPanelPlugin {
    fn build(&self, app : &mut App) {
        app.add_systems(Startup,setup_widget)
        .add_systems(PostUpdate,update_widget_system);
    }
}


fn setup_widget(
    mut commands: Commands,
) {
    let text_style = TextStyle {
        font_size: UiConsts::STANDARD_UI_FONT_SIZE,
        color: Color::WHITE,
        ..default()
    };

    commands.spawn((
        SelectionPanel,
        NodeBundle {
            background_color: BackgroundColor(Color::BLACK.with_alpha(1.0)),
            border_color : BorderColor(Color::srgb(0.1,0.1,0.2)),
            z_index: ZIndex::Global(i32::MAX-1),
            style: Style {
                flex_direction : FlexDirection::Column,
                align_items : AlignItems::FlexStart,
                position_type: PositionType::Absolute,
                justify_content : JustifyContent::Center,
                max_width : Val::Percent(20.),
                width: Val::Auto,
                height: Val::Auto,
                left: Val::Percent(1.),
                bottom: Val::Auto,
                top: Val::Percent(25.),
                right: Val::Auto,
                border : UiRect::all(Val::Px(4.0)),
                padding: UiRect::all(Val::Px(1.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        NoDeselect
    ))
    .with_children(|parent| {
        for i in 0..GalaxyConfig::MAX_SYSTEM_BODIES {
            parent.spawn((
                SelectionPanelTabRoot { slot : i as i32},
                SelectionProxy::new(InterfaceIdentifier::CurrentSystemOrbiter(i as u32)),
                super::UiSelectionHighlight,
                ButtonBundle {
                    background_color : Color::srgb(0.0,0.0,0.0).into(),
                    z_index: ZIndex::Global(i32::MAX),
                    style: Style {
                        flex_direction : FlexDirection::Column,
                        align_items : AlignItems::FlexStart,
                        position_type: PositionType::Relative,
                        justify_content : JustifyContent::FlexStart,
                        width: Val::Auto,//(100.),
                        border : UiRect::all(Val::Px(4.0)),
                        padding: UiRect::all(Val::Px(2.0)),
                        margin : UiRect::all(Val::Px(1.0)),
                        height : Val::Auto,    
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ))
            .with_children(|parent| {
                let label = format!("Tab {}  ", i.to_string());
                parent.spawn((
                    SelectionPanelTabHeader { slot : i as i32},
                    TextBundle {
                        background_color : Color::srgba(0.2,0.2,0.2, 0.5).into(),
                        text: Text::from_sections([
                            TextSection {
                                value: label.into(),
                                style: text_style.clone()
                            },
                            TextSection {
                                value: " N/A".into(),
                                style : text_style.clone()
                            },
                        ]),
                        ..Default::default()
                    },
                    Pickable::IGNORE,
                ));
                parent.spawn((
                    SelectionPanelTabDetails { slot : i as i32},
                    TextBundle {
                        background_color : Color::srgb(0.2,0.2,0.2).into(),
                        text: Text::from_sections([
                            TextSection {
                                value: "".into(),
                                style : text_style.clone()
                            },
                            TextSection {
                                value: " N/A".into(),
                                style : text_style.clone()
                            },
                        ]),
                        ..Default::default()
                    },
                    Pickable::IGNORE,
                ));
            });
        }
    });
}

use crate::simulation::fleet_behaviour::navigation::{NavPosition,Navigator,Action,Plan};
use crate::galaxy::fleet::{Fleet,FleetColonyCrew};

fn update_widget_system(
    mut root_query: Query<(&mut Style, &mut BackgroundColor, &SelectionPanelTabRoot)>,
    mut header_query: Query<(&mut Text,&SelectionPanelTabHeader), Without<SelectionPanelTabRoot>>,
    mut details_query: Query<(&mut Text, &mut Style, &SelectionPanelTabDetails), (Without<SelectionPanelTabRoot>,Without<SelectionPanelTabHeader>)>,
    fleet_query : Query<(&Fleet,&FleetColonyCrew,&NavPosition,&Navigator)>,
    empires_query : Query<&Empire>,
    selection : Res<Selection>,
    description_query : Query<&Description>,
) {
    if selection.is_changed() {
        let Some(selected) = selection.selected else {            
            for (mut style,_,_) in root_query.iter_mut() {
                style.display = Display::None;
            }
            return;
        };
        let Ok((fleet,colony_crew,nav_pos,navigator)) = fleet_query.get(selected) else {
            for (mut style,_,_) in root_query.iter_mut() {
                style.display = Display::None;
            }
            return; 
        };
        let len = 1;

        for (mut text, panel) in header_query.iter_mut() {
            if panel.slot < len {
                text.sections[0].value = format!("{} ", "Unnamed Fleet");
                text.sections[0].style.color = Color::WHITE;

                text.sections[1].value = format!("({})", "Fleet");
                text.sections[1].style.color = Color::linear_rgb(0.5,0.5,0.5);
            }
        }
        for (mut text, mut style, panel) in details_query.iter_mut() {
            if panel.slot < len {
                style.display = Display::None;
                    // try grab colony

                let dest_name = match colony_crew.destination {
                    None => "None",
                    Some(entity) => {
                        let planet_desc = description_query.get(entity).unwrap();
                        &planet_desc.name
                    }
                };

                let action_name = match navigator.action {
                    Action::Jumping => "Jumping",
                    Action::BeingDestroyed => "BeingDestroyed",
                    Action::Colonise(_) => "Colonising",
                    Action::Idle => "Idle",
                    Action::Move(_) => "Moving",
                };

                let plan_name = match navigator.plan_queue.last() {
                    None => "None",
                    Some(plan) => match plan {
                        Plan::Colonise(_) => "Colonise",
                        Plan::Jump(_) => "Jump",
                        Plan::ReachHomeEmpire => "ReachHomeEmpire",
                        Plan::ReachPoint(_) => "ReachPoint",
                        Plan::ReachSystem(_) => "ReachSystem",
                    }
                };
                
                text.sections[1].value = format!("Fleet Size {}\nCurrent Action: {}\nNext Plan: {} (remaining steps: {})\nColony Ship Destination: {} (crew {})", 1, action_name, plan_name, navigator.plan_queue.len(),dest_name, colony_crew.colonists);

                text.sections[0].style.color = Color::srgb(0.25,0.25,1.0);
                text.sections[1].style.color = Color::srgb(0.25,0.25,1.0);

                style.display = Display::Flex;
            }
        }
        for (mut style, mut bg, panel) in root_query.iter_mut() {
            if panel.slot < len {
                let empire = empires_query.get(fleet.owner).unwrap();
                *bg = empire.color.into();
                                          
                style.display = Display::Flex;
            } else {
                style.display = Display::None;
            }
        }
    }
}