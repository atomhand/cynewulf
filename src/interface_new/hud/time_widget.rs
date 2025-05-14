use bevy::diagnostic::DiagnosticsStore;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

pub struct TimeWidgetPlugin;

impl Plugin for TimeWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_widget)
            .add_systems(Update, update_widget_system)
            .add_plugins(FrameTimeDiagnosticsPlugin::default());
    }
}

#[derive(Component)]
struct SimulationWidget {
    ui_slot: i32,
}

fn setup_widget(mut commands: Commands) {
    let holder = commands
        .spawn((
            //FpsRoot,
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::FlexStart,
                width: Val::Px(240.),
                right: Val::Percent(1.),
                top: Val::Percent(1.),
                bottom: Val::Auto,
                left: Val::Auto,
                padding: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::linear_rgba(0.0, 0.03, 0.08, 0.5)),
            GlobalZIndex(i32::MAX - 1),
        ))
        .id();

    for i in 0..3 {
        // create our UI root node
        // this is the wrapper/container for the text
        let root = commands
            .spawn((
                Node {
                    padding: UiRect::all(Val::Px(4.0)),
                    ..Default::default()
                },
                GlobalZIndex(i32::MAX),
            ))
            .id();
        let text_fps = commands
            .spawn((
                SimulationWidget { ui_slot: i },
                Text(" N/A".to_string()),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
            ))
            .id();
        commands.entity(holder).add_child(root);
        commands.entity(root).add_child(text_fps);
    }
}
use crate::simulation::SimTime;
use crate::simulation::{SimulationMode, SimulationSettings};

fn update_widget_system(
    sim_time: Res<SimTime>,
    sim_settings: Res<SimulationSettings>,
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<(&mut Text, &SimulationWidget)>,
) {
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0);
    let frame_time = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0);
    let fps_str = format!("{fps:.1} ({frame_time:.2} ms)");

    let (day, month, year) = sim_time.to_daymonthyear();
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

    for (mut text, widget) in &mut query {
        let label = match widget.ui_slot {
            0 => "Date:",
            1 => "Speed:",
            2 => "FPS:",
            _ => "Error:",
        };
        let text_val = match widget.ui_slot {
            0 => format!("{:2}/{:2}/{:2}", day, month, year),
            1 => speed.into(),
            2 => fps_str.clone(),
            _ => "".into(),
        };

        text.0 = format!("{:>6} {:<8}", label, text_val); //format!("{value:>4.0}");
    }
}
