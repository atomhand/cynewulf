use bevy::prelude::*;

/// Marker to find the text entity so we can update it
#[derive(Component)]
struct HoverWidget;

pub struct HoverWidgetPlugin;

impl Plugin for HoverWidgetPlugin {
    fn build(&self, app : &mut App) {
        app.add_systems(Startup,setup_widget)
        .add_systems(Update,update_widget_system);
    }
}

fn setup_widget(
    mut commands: Commands,
) {
    let holder = commands.spawn((
        //FpsRoot,
        NodeBundle {
            // give it a dark background for readability
            background_color: BackgroundColor(Color::BLACK.with_alpha(0.5)),
            // make it "always on top" by setting the Z index to maximum
            // we want it to be displayed over all other UI
            z_index: ZIndex::Global(i32::MAX),
            style: Style {
                flex_direction : FlexDirection::Column,
                align_items : AlignItems::FlexStart,
                position_type: PositionType::Absolute,
                justify_content : JustifyContent::FlexStart,
                width: Val::Px(256.),
                //height: Val::Px(48.),
                // position it at the top-right corner
                // 1% away from the top window edge
                left: Val::Percent(1.),
                top: Val::Percent(1.),
                // set bottom/left to Auto, so it can be
                // automatically sized depending on the text
                bottom: Val::Auto,
                right: Val::Auto,
                // give it some padding for readability
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    // create our UI root node
    // this is the wrapper/container for the text
    let root = commands.spawn((
        NodeBundle {
            // give it a dark background for readability
            background_color: BackgroundColor(Color::BLACK.with_alpha(0.5)),
            // make it "always on top" by setting the Z index to maximum
            // we want it to be displayed over all other UI
            z_index: ZIndex::Global(i32::MAX),
            style: Style {
                // give it some padding for readability
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();
    // create our text
    let label = "Hovered: ";
    let text_fps = commands.spawn((
        HoverWidget,
        TextBundle {
            // use two sections, so it is easy to update just the number
            text: Text::from_sections([
                TextSection {
                    value: label.into(),
                    style: TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        // if you want to use your game's font asset,
                        // uncomment this and provide the handle:
                        // font: my_font_handle
                        ..default()
                    }
                },
                TextSection {
                    value: " N/A".into(),
                    style: TextStyle {
                        font_size: 16.0,
                        color: Color::WHITE,
                        // if you want to use your game's font asset,
                        // uncomment this and provide the handle:
                        // font: my_font_handle
                        ..default()
                    }
                },
            ]).with_no_wrap(),
            ..Default::default()
        },
    )).id();
    commands.entity(holder).push_children(&[root]);
    commands.entity(root).push_children(&[text_fps]);
}

use crate::galaxy::Selection;
use crate::galaxy::description::Description;

fn update_widget_system(
    selected : Res<Selection>,
    description_query : Query<&Description, Without<HoverWidget>>,
    mut query: Query<&mut Text, With<HoverWidget>>,
) {

    let hovered = if let Some(entity) = selected.hovered {
        if let Ok(description) = description_query.get(entity) {
            description.name.clone()
        } else {
            "No Valid Name".into()
        }
    } else {
        "None".into()
    };

    if let Ok(mut text) = query.get_single_mut() {
        let text_val = hovered;

        text.sections[1].value = text_val; //format!("{value:>4.0}");
        text.sections[1].style.color = Color::srgb(0.0, 0.5, 1.0);
    }
}