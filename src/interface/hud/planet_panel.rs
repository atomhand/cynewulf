use bevy::prelude::*;

struct PlanetPanelState {
    planet : Entity,
    ui_root : Entity,
}

struct PlanetPanelPlugin;

impl Plugin for PlanetPanelPlugin {
    fn build(&self, app : &mut App) {
        app.add_systems(Startup,build_planet_panel)
            .add_systems(Update,update_planet_panel);
    }
}

fn build_planet_panel(mut commands : Commands) {
    commands.spawn(
        NodeBundle {
            style : Style {
                ..default()
            },
            ..default()
        }
    )
    .with_children(
        |parent| {

        }
    );
}

fn update_planet_panel(

) {

}