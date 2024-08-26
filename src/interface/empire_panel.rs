use bevy::prelude::*;
use crate::galaxy::{Selection,GalaxyConfig};
use bevy_mod_picking::prelude::*;
use crate::prelude::*;

/// Marker to find the text entity so we can update it
#[derive(Component)]
struct EmpirePanel {
    title_text : Entity,
    button : Entity,
}

pub struct EmpirePanelPlugin;

impl Plugin for EmpirePanelPlugin {
    fn build(&self, app : &mut App) {
        app.add_systems(Startup,setup_widget)
        .add_systems(Update,update_widget_system);
    }
}

fn quick_text(font_size : f32, text : String) -> TextSection {
    TextSection {
        value: text,
        style: TextStyle {
            font_size: font_size,
            color: Color::WHITE,
            ..default()
        }
    }
}

fn setup_widget(
    mut commands: Commands,
) {
    let label = commands.spawn((
        TextBundle {
            background_color : Color::srgba(0.2,0.2,0.2, 0.5).into(),
            text: Text::from_sections([
                quick_text(16.0,"".into()),
                quick_text(16.0, " Empire".into()),
            ]),
            ..Default::default()
        },
        Pickable::IGNORE,
    )).id();

    let button = commands.spawn(
        ButtonBundle {
            background_color : Color::srgb(0.0,0.0,0.0).into(),
            z_index: ZIndex::Global(i32::MAX),
            style: Style {
                flex_direction : FlexDirection::Column,
                align_items : AlignItems::FlexStart,
                position_type: PositionType::Relative,
                justify_content : JustifyContent::FlexStart,
                width: Val::Px(256.),
                border : UiRect::all(Val::Px(4.0)),
                padding: UiRect::all(Val::Px(2.0)),
                margin : UiRect::all(Val::Px(1.0)),
                height : Val::Auto,    
                ..Default::default()
            },
            ..Default::default()
        },
    ).push_children(&[label]).id();

    commands.spawn((
        EmpirePanel {
            title_text : label,
            button : button
        },
        NodeBundle {
            background_color: BackgroundColor(Color::BLACK.with_alpha(1.0)),
            z_index: ZIndex::Global(i32::MAX-1),
            style: Style {
                flex_direction : FlexDirection::Column,
                align_items : AlignItems::FlexEnd,
                position_type: PositionType::Absolute,
                justify_content : JustifyContent::Center,
                width: Val::Auto,
                height: Val::Auto,
                left: Val::Percent(1.),
                bottom: Val::Auto,
                top: Val::Percent(1.),
                right: Val::Auto,
                padding: UiRect::all(Val::Px(1.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    )).push_children(&[button]);
}

fn update_widget_system(
    mut label_query: Query<&mut Text>,
    mut bg_query: Query<&mut BackgroundColor,(Without<Text>,Without<EmpirePanel>)>,
    panel_query : Query<&EmpirePanel,Without<Text>>,
    empires_query : Query<&Empire, (Without<Text>,Without<BackgroundColor>)>,
    player_empire : Res<crate::galaxy::empire::PlayerEmpire>
) {
    if player_empire.is_changed() {
        let Some(empire) = player_empire.empire else { return; };
        let Ok(empire) = empires_query.get(empire) else { return; };

        let panel = panel_query.single();
        let Ok(mut bg) = bg_query.get_mut(panel.button) else { return; };
        let Ok(mut text) = label_query.get_mut(panel.title_text) else { return; };

        text.sections[0].value = empire.name.clone();
        *bg = empire.color.into();
    }
}