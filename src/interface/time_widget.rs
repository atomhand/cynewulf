use bevy::prelude::*;

pub struct TimeWidgetPlugin;

impl Plugin for TimeWidgetPlugin {
    fn build(&self, app : &mut App) {
        app.add_systems(Startup, setup_widget)
            .add_systems(Update,update_widget_system);
    }
}

#[derive(Component)]
struct SimulationWidget {
    ui_slot : i32
}

fn setup_widget(
    mut commands: Commands,
) {
    let holder = commands.spawn((
        //FpsRoot,
        NodeBundle {
            // give it a dark background for readability
            background_color: BackgroundColor(Color::BLACK.with_alpha(1.0)),
            // make it "always on top" by setting the Z index to maximum
            // we want it to be displayed over all other UI
            z_index: ZIndex::Global(i32::MAX-1),
            style: Style {
                flex_direction : FlexDirection::Column,
                align_items : AlignItems::FlexStart,
                position_type: PositionType::Absolute,
                justify_content : JustifyContent::FlexStart,
                width: Val::Px(160.),
                //height: Val::Px(48.),
                // position it at the top-right corner
                // 1% away from the top window edge
                right: Val::Percent(1.),
                top: Val::Percent(1.),
                // set bottom/left to Auto, so it can be
                // automatically sized depending on the text
                bottom: Val::Auto,
                left: Val::Auto,
                // give it some padding for readability
                padding: UiRect::all(Val::Px(4.0)),
                ..Default::default()
            },
            ..Default::default()
        },
    )).id();

    for i in 0..2 {
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
        let label = match i {
            0 => "Date: ",
            1 => "Speed: ",
            _ => "Error: "
        };
        let text_fps = commands.spawn((
            SimulationWidget { ui_slot : i},
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
                ]),
                ..Default::default()
            },
        )).id();
        commands.entity(holder).push_children(&[root]);
        commands.entity(root).push_children(&[text_fps]);
    }
}

use crate::simulation::SimTime;
use crate::simulation::{SimulationMode, SimulationSettings};

fn update_widget_system(
    sim_time : Res<SimTime>,
    sim_settings : Res<SimulationSettings>,
    mut query: Query<(&mut Text, &SimulationWidget)>,
) {
    let (day,month,year) = sim_time.to_daymonthyear();
    //let dmy = format!("{}/{}/{}", day, month, year);
    let speed = if sim_settings.paused {
        "Paused"
    } else {
        match sim_settings.mode {
            SimulationMode::Slow => "Slow",        
            SimulationMode::Normal => "Normal",
            SimulationMode::Fast => "Fast",
            SimulationMode::Fastest => "Fastest",
        }
    };

    for (mut text,widget) in &mut query {
        let text_val = match widget.ui_slot {
            0 => format!("{}/{}/{}", day, month, year),
            1 => speed.into(),
            _ => "".into()
        };

        text.sections[1].value = text_val; //format!("{value:>4.0}");
        text.sections[1].style.color = Color::srgb(0.0, 1.0, 0.0);
    }
}