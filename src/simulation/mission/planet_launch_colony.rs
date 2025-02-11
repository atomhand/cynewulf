use crate::prelude::*;
use bevy::prelude::*;

pub struct PlanetAutoColonyMissionPlugin;
use super::super::SimTick;

impl Plugin for PlanetAutoColonyMissionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, add_mission_system)
            .add_systems(SimTick, update_mission_system);
    }
}

fn add_mission_system(query: Query<Entity, Added<Colony>>, mut commands: Commands) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(LaunchColonyMission::default());
    }
}

#[derive(Component)]
pub struct LaunchColonyMission {
    target: Option<Entity>,
    efficiency_factor: i32,
    target_crew_size: i32,
    target_capital: i32,
    current_crew: i32,
    current_capital: i32,
}

impl Default for LaunchColonyMission {
    fn default() -> Self {
        Self {
            target: None,
            efficiency_factor: 1,
            target_crew_size: 10000,
            target_capital: 10000,
            current_crew: 0,
            current_capital: 0,
        }
    }
}

fn update_mission_system(
    mut query: Query<(&Planet, &mut LaunchColonyMission, &Colony)>,
    planet_query: Query<(&Planet, Option<&Colony>)>,
    mut commands: Commands,
) {
    for (origin_planet, mut mission, origin_colony) in query.iter_mut() {
        let target_valid = true;

        /*
        mission.target
            .and_then(|target|
                planet_query.get(target).ok()
                .and_then(
                    |(planet,colony)|
                    is_valid_target(origin_colony, planet, colony)
                )
            ).is_some();
        */

        if target_valid {
            mission.current_crew += origin_colony.get_daily_colonists() as i32;
            mission.current_capital += origin_colony.get_daily_colony_ship_construction() as i32;

            // LAUNCH THE COLONY SHIP
            if mission.current_crew >= mission.target_crew_size
                && mission.current_capital >= mission.target_capital
            {
                commands.spawn((
                    crate::galaxy::fleet::FleetBundle::new(
                        origin_colony.owner,
                        origin_planet.system_local_pos(),
                        origin_planet.star_id,
                    ),
                    crate::galaxy::fleet::FleetColonyCrew {
                        destination: mission.target,
                        colonists: mission.current_crew as i64,
                    },
                ));
                mission.current_crew = 0;
                mission.current_capital = 0;
            }
        } else {
            // TRY FIND A NEW TARGET?
        }
    }
}
