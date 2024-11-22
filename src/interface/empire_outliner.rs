use bevy::prelude::*;
use crate::galaxy::Selection;
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
    slot : i32,
    material : Handle<StarIconMaterial>
}
#[derive(Component)]
struct SelectionPanelTabDetails {
    slot : i32
}

pub struct EmpireOutlinerPlugin;

impl Plugin for EmpireOutlinerPlugin {
    fn build(&self, app : &mut App) {
        app.add_systems(Startup,setup_widget)
        .add_systems(PostUpdate,update_widget_system);
    }
}

use crate::graphics::StarIconMaterial;

fn setup_widget(
    mut commands: Commands,
    mut materials : ResMut<Assets<StarIconMaterial>>,
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
                align_items : AlignItems::FlexEnd,
                position_type: PositionType::Absolute,
                justify_content : JustifyContent::Start,
                max_width : Val::Percent(20.),
                width: Val::Px(256.),
                height: Val::Percent(50.),
                overflow: Overflow::clip_y(),
                left: Val::Auto,
                bottom: Val::Auto,
                top: Val::Px(96.+16.),
                right: Val::Percent(1.),
                border : UiRect::all(Val::Px(4.0)),
                padding: UiRect::all(Val::Px(1.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        NoDeselect
    ))
    .with_children(|parent| {
        parent.spawn((
            super::UiSelectionHighlight,
            ButtonBundle {
                border_color : Color::srgb(1.,1.,1.).into(),
                background_color : Color::srgb(0.1,0.1,0.2).into(),
                z_index: ZIndex::Global(i32::MAX),
                style: Style {
                    flex_direction : FlexDirection::Column,
                    align_items : AlignItems::Center,
                    position_type: PositionType::Relative,
                    justify_content : JustifyContent::Center,
                    width: Val::Percent(100.),//(100.),
                    border : UiRect::all(Val::Px(2.0)),
                    padding: UiRect::all(Val::Px(4.0)),
                    margin : UiRect::all(Val::Px(1.0)),
                    height : Val::Auto,    
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle {
                    background_color : Color::srgba(0.2,0.2,0.2, 0.5).into(),
                    text: Text::from_sections([
                        TextSection {
                            value: "IMPERIAL SYSTEMS".into(),
                            style: text_style.clone()
                        },
                        TextSection {
                            value: "".into(),
                            style : text_style.clone()
                        },
                    ]),
                    ..Default::default()
                },
                Pickable::IGNORE,
            ));
        });
        for i in 0..100 {
            parent.spawn((
                SelectionPanelTabRoot { slot : i as i32},
                SelectionProxy::new(InterfaceIdentifier::EmpireStar(i as u32)),
                super::UiSelectionHighlight,
                ButtonBundle {
                    background_color : Color::srgb(0.0,0.0,0.0).into(),
                    z_index: ZIndex::Global(i32::MAX),
                    style: Style {
                        flex_direction : FlexDirection::Row,
                        align_items : AlignItems::Center,
                        position_type: PositionType::Relative,
                        justify_content : JustifyContent::FlexStart,
                        width: Val::Percent(100.),//(100.),
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
                let mat = materials.add(StarIconMaterial {
                    //radius : 1.0,
                    color : Vec4::splat(1.0)
                });
                parent.spawn(MaterialNodeBundle {
                    style : Style {
                        width: Val::Px(32.0),
                        height : Val::Px(32.0),
                        .. default()
                    },
                    material : mat.clone(),
                    ..default()
                });
                let label = format!("Tab {}  ", i.to_string());
                parent.spawn((
                    SelectionPanelTabHeader { slot : i as i32, material : mat },
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
    star_query : Query<(&Description,&SystemIndex,&Star), Without<SelectionPanelTabHeader>>,
    player_empire : Res<PlayerEmpire>,
    empires_index : Query<&EmpireIndex>,
    mut star_icon_materials : ResMut<Assets<StarIconMaterial>>,
) {
    if selection.is_changed() {
        let Some(empire) = player_empire.empire else { 
            for (mut style,_,_) in root_query.iter_mut() {
                style.display = Display::None;
            }
            return
        };

        let Ok(empire_index) = empires_index.get(empire) else { return; };
        let empire_stars = &empire_index.systems;
        let desc = empire_stars.iter().map(|x| star_query.get(*x).unwrap()).collect::<Vec<_>>();
        let len = empire_stars.len() as i32;

        for (mut text, panel) in header_query.iter_mut() {
            if panel.slot < len {                
                let star_mat = star_icon_materials.get_mut(&panel.material).unwrap();
                star_mat.color = Vec4::from((desc[panel.slot as usize].2.get_color(),1.0));

                text.sections[0].value = format!("{} ", desc[panel.slot as usize].0.name);
                text.sections[0].style.color = Color::WHITE;
                text.sections[1].value = format!("({})", desc[panel.slot as usize].1.population.format_big_number());
                text.sections[1].style.color = desc[panel.slot as usize].0.type_color();
            }
        }
        for (_text, mut style, panel) in details_query.iter_mut() {
            if panel.slot < len {
                style.display = Display::None;
                /*
                if Some(empire_stars[panel.slot as usize]) == selection.selected_system {
                    text.sections[1].value = format!("Panel Details for Star {}", desc[panel.slot as usize].0.name);
                    text.sections[0].style.color = Color::srgb(0.25,0.25,1.0);
                    text.sections[1].style.color = Color::srgb(0.25,0.25,1.0);
                    style.display = Display::Flex;
                }
                */
            }
        }
        for (mut style, mut bg, panel) in root_query.iter_mut() {
            if panel.slot < len {
                *bg = desc[panel.slot as usize].0.empire_color.unwrap_or(
                    Color::srgb(0.1,0.1,0.1)).into();                                          
                style.display = Display::Flex;
            } else {
                style.display = Display::None;
            }
        }
    }
}