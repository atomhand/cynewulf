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

pub struct SelectionPanelPlugin;

impl Plugin for SelectionPanelPlugin {
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
                bottom: Val::Percent(1.),
                top: Val::Auto,
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

fn update_widget_system(
    mut root_query: Query<(&mut Style, &mut BackgroundColor, &SelectionPanelTabRoot)>,
    mut header_query: Query<(&mut Text,&SelectionPanelTabHeader), Without<SelectionPanelTabRoot>>,
    mut details_query: Query<(&mut Text, &mut Style, &SelectionPanelTabDetails), (Without<SelectionPanelTabRoot>,Without<SelectionPanelTabHeader>)>,
    selection : Res<Selection>,
    description_query : Query<&Description, Without<SelectionPanelTabHeader>>,
    star_query : Query<&Star, Without<SelectionPanelTabHeader>>,
    planet_colony_query : Query<(&Planet,Option<&Colony>), Without<SelectionPanelTabHeader>>,
) {
    if selection.is_changed() {
        let Some(star_entity) = selection.selected_system else {            
            for (mut style,_,_) in root_query.iter_mut() {
                style.display = Display::None;
            }
            return;
        };
        let Ok(star) = star_query.get(star_entity) else { return; };
        let star_and_orbiters = &star.orbiters;
        /*
        let mut star_and_orbiters = Vec::<Entity>::new();
        if let Some(star_entity) = selection.selected_system {
            if let Ok(star) = star_query.get(star_entity) {
                star_and_orbiters.insert(0, star_entity);
                star_and_orbiters.extend_from_slice(star.orbiters.as_slice());
            }
        }
        */

        let desc = star_and_orbiters.iter().map(|x| description_query.get(*x).unwrap()).collect::<Vec<_>>();

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
                if Some(star_and_orbiters[panel.slot as usize]) == selection.selected {
                    // try grab colony
                    text.sections[1].value = "".to_string();
                    text.sections[0].value = format!("Panel Details for planet {}", desc[panel.slot as usize].name);
                    if let Ok((planet,colony)) = planet_colony_query.get(star_and_orbiters[panel.slot as usize]) {
                        text.sections[0].value = format!("Size: {} | Insolation: {}\n", planet.radius, planet.insolation);
                        if let Some(colony) = colony {
                            text.sections[1].value = format!("{}\n\n{}", colony.population.details(), colony.economy);
                        }
                    }

                    text.sections[0].style.color = Color::srgb(0.25,0.25,1.0);
                    text.sections[1].style.color = Color::srgb(0.25,0.25,1.0);

                    style.display = Display::Flex;
                }
            }
        }
        for (mut style, mut bg, panel) in root_query.iter_mut() {
            if panel.slot < len {
                *bg = desc[panel.slot as usize].empire_color.unwrap_or(
                    Color::srgb(0.1,0.1,0.1)).into();
                                          
                style.display = Display::Flex;

                /*
                *border_color = if Some(star_and_orbiters[panel.slot as usize]) == selection.hovered {
                    if Some(star_and_orbiters[panel.slot as usize]) == selection.selected {
                        Color::srgb(1.0,80./255.,0.)
                    } else {
                        Color::WHITE
                    }
                } else if Some(star_and_orbiters[panel.slot as usize]) == selection.selected {
                    Color::srgb(1.0,165./255.,0.)
                } else {
                    Color::srgb(0.1,0.1,0.1)
                }.into();
                */
            } else {
                style.display = Display::None;
            }
        }
    }
}