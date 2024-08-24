use bevy::prelude::*;
use crate::galaxy::{Selection,GalaxyConfig};
use bevy_mod_picking::prelude::*;

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


pub struct SelectionPanelPlugin;

impl Plugin for SelectionPanelPlugin {
    fn build(&self, app : &mut App) {
        app.add_systems(Startup,setup_widget)
        .add_systems(Update,(widget_interact_system,update_widget_system).chain());
    }
}

fn setup_widget(
    mut commands: Commands,
) {
    commands.spawn((
        SelectionPanel,
        NodeBundle {
            // give it a dark background for readability
            background_color: BackgroundColor(Color::BLACK.with_alpha(1.0)),
            // make it "always on top" by setting the Z index to maximum
            // we want it to be displayed over all other UI
            z_index: ZIndex::Global(i32::MAX-1),
            style: Style {
                flex_direction : FlexDirection::Column,
                align_items : AlignItems::FlexEnd,
                position_type: PositionType::Absolute,
                justify_content : JustifyContent::Center,
                width: Val::Auto,
                height: Val::Auto,
                //height: Val::Px(48.),
                // position it at the top-right corner
                // 1% away from the top window edge
                left: Val::Percent(1.),
                bottom: Val::Percent(1.),
                top: Val::Auto,
                right: Val::Auto,
                // set bottom/left to Auto, so it can be
                // automatically sized depending on the text
                // give it some padding for readability
                padding: UiRect::all(Val::Px(1.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    ))
    .with_children(|parent| {
        for i in 0..GalaxyConfig::MAX_SYSTEM_BODIES {
            parent.spawn((
                SelectionPanelTabRoot { slot : i as i32},
                ButtonBundle {
                    // give it a dark background for readability
                    background_color : Color::srgb(0.0,0.0,0.0).into(),
                    // make it "always on top" by setting the Z index to maximum
                    // we want it to be displayed over all other UI
                    z_index: ZIndex::Global(i32::MAX),
                    style: Style {
                        flex_direction : FlexDirection::Column,
                        align_items : AlignItems::FlexStart,
                        position_type: PositionType::Relative,
                        justify_content : JustifyContent::FlexStart,
                        width: Val::Px(256.),
                        border : UiRect::all(Val::Px(4.0)),
                        // give it some padding for readability
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
                        // use two sections, so it is easy to update just the number
                        text: Text::from_sections([
                            TextSection {
                                value: label.into(),
                                style: TextStyle {
                                    font_size: 16.0,
                                    color: Color::WHITE,
                                    ..default()
                                }
                            },
                            TextSection {
                                value: " N/A".into(),
                                style: TextStyle {
                                    font_size: 16.0,
                                    color: Color::WHITE,
                                    ..default()
                                }
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
                        // use two sections, so it is easy to update just the number
                        text: Text::from_sections([
                            TextSection {
                                value: "".into(),
                                style: TextStyle {
                                    font_size: 16.0,
                                    color: Color::WHITE,
                                    ..default()
                                }
                            },
                            TextSection {
                                value: " N/A".into(),
                                style: TextStyle {
                                    font_size: 16.0,
                                    color: Color::WHITE,
                                    ..default()
                                }
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

use crate::galaxy::{Star,Description};

fn widget_interact_system(
    mut root_query: Query<(Option<&PickingInteraction>, &SelectionPanelTabRoot)>,
    mut selection : ResMut<Selection>,
    star_query : Query<&Star, Without<SelectionPanelTabHeader>>,
) {
    let mut star_and_orbiters = Vec::<Entity>::new();
    if let Some(star_entity) = selection.selected_system {
        if let Ok(star) = star_query.get(star_entity) {
            star_and_orbiters.insert(0, star_entity);
            star_and_orbiters.extend_from_slice(star.orbiters.as_slice());
        }
    }

    for(interaction, panel) in &mut root_query {
        match interaction {
            Some(PickingInteraction::Pressed) => {
                if panel.slot < star_and_orbiters.len() as i32 {
                    selection.selected = Some(star_and_orbiters[panel.slot as usize]);
                    selection.selected_system = Some(star_and_orbiters[0]);
                }
            }
            Some(PickingInteraction::Hovered) => {
                if panel.slot < star_and_orbiters.len() as i32 {
                    selection.hovered = Some(star_and_orbiters[panel.slot as usize]);
                }
            }
            Some(PickingInteraction::None) | None => {}
        }
    }
}

use crate::galaxy::Planet;
use crate::simulation::data::Colony;

fn update_widget_system(
    mut root_query: Query<(&mut Style, &mut BackgroundColor, &mut BorderColor, &SelectionPanelTabRoot)>,
    mut header_query: Query<(&mut Text,&SelectionPanelTabHeader), Without<SelectionPanelTabRoot>>,
    mut details_query: Query<(&mut Text, &mut Style, &SelectionPanelTabDetails), (Without<SelectionPanelTabRoot>,Without<SelectionPanelTabHeader>)>,
    selection : Res<Selection>,
    description_query : Query<&Description, Without<SelectionPanelTabHeader>>,
    star_query : Query<&Star, Without<SelectionPanelTabHeader>>,
    planet_colony_query : Query<(&Planet,Option<&Colony>), Without<SelectionPanelTabHeader>>,
) {

    if selection.is_changed() {

        let mut star_and_orbiters = Vec::<Entity>::new();
        if let Some(star_entity) = selection.selected_system {
            if let Ok(star) = star_query.get(star_entity) {
                star_and_orbiters.insert(0, star_entity);
                star_and_orbiters.extend_from_slice(star.orbiters.as_slice());
            }
        }

        let mut desc = Vec::<&Description>::new();

        for entity in &star_and_orbiters {
            desc.push(description_query.get(*entity).unwrap());
        }
        let len = star_and_orbiters.len() as i32;

        for (mut text, panel) in header_query.iter_mut() {
            if panel.slot < len {

                text.sections[0].value = format!("{} ", desc[panel.slot as usize].name);
                text.sections[0].style.color = Color::WHITE;

                text.sections[1].value = format!("({})", desc[panel.slot as usize].type_name());
                text.sections[1].style.color = desc[panel.slot as usize].type_color();

                if let Ok((_planet,colony)) = planet_colony_query.get(star_and_orbiters[panel.slot as usize]) {
                    if let Some(colony) = colony {
                        text.sections[1].value = format!("({}, {})", colony.population.to_string(), desc[panel.slot as usize].type_name());
                    }
                }
            }
        }
        for (mut text, mut style, panel) in details_query.iter_mut() {
            if panel.slot < len {
                style.display = Display::None;
                if let Some(selected) = selection.selected {
                    if star_and_orbiters[panel.slot as usize] == selected {
                        // try grab colony
                        text.sections[1].value = format!("Panel Details for planet {}", desc[panel.slot as usize].name);
                        if let Ok((_planet,colony)) = planet_colony_query.get(star_and_orbiters[panel.slot as usize]) {
                            if let Some(colony) = colony {
                                text.sections[1].value = format!("{}", colony.economy);
                            }
                        }

                        text.sections[0].style.color = Color::srgb(0.25,0.25,1.0);
                        text.sections[1].style.color = Color::srgb(0.25,0.25,1.0);

                        style.display = Display::Flex;
                    }
                }
            }
        }
        for (mut style, mut bg, mut border_color, panel) in root_query.iter_mut() {
            if panel.slot < len {
                let mut col = Color::srgb(0.1,0.1,0.1);

                *bg = if let Some(owner_color) = desc[panel.slot as usize].empire_color {
                    owner_color.into()
                } else {
                    Color::srgb(0.1,0.1,0.1).into()
                };


                if let Some(hovered) = selection.hovered {
                    if star_and_orbiters[panel.slot as usize] == hovered {
                        col = Color::WHITE;
                    }
                }
                if let Some(selected) = selection.selected {
                    if star_and_orbiters[panel.slot as usize] == selected {
                        col = Color::srgb(1.0,165./255.,0.);
                    }
                }

                style.display = Display::Flex;

                *border_color = col.into();

                //*visibility = Visibility::Visible;
            } else {
                style.display = Display::None;
                //*visibility = Visibility::Hidden;
            }
            //style.height = Val::Auto;
        }
    }
}