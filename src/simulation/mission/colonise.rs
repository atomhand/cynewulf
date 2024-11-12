use bevy::prelude::*;
use crate::prelude::*;

use super::MissionState;

#[derive(Component)]
struct ColoniseMission {
    target : Entity,
    origin_empire : Entity,
    mission_state : MissionState
}

fn validate_missions_system(
    mut mission_query : Query<&mut ColoniseMission>,
    planet_query : Query<(&Planet,Option<&Colony>)>,
    star_query : Query<&StarClaim>,
    empires_query : Query<&Empire>,
    hypernet : Res<Hypernet>
) {
    for mut mission in mission_query.iter_mut() {
        if empires_query.get(mission.origin_empire).is_err() {
            mission.mission_state = MissionState::FailedError;
            continue;
        };
        let Ok((planet,colony)) = planet_query.get(mission.target) else {
            mission.mission_state = MissionState::FailedError;
            continue;
        };

        let star= star_query.get(hypernet.star(planet.star_id).entity).unwrap(); // Stars do not simply disappear, so this is a crash

        if let Some(colony) = colony {
            // Mission fails if you are beaten to it by another empire
            // POINT FOR IMPROVEMENT:
            // Should maybe check if the mission-originating empire ~knows~ that it's been beaten to it
            if star.owner != Some(mission.origin_empire) || colony.owner != mission.origin_empire {
                mission.mission_state = MissionState::Failed;
            } else {
                mission.mission_state = MissionState::Succeeded;
            }
        }
    }
}
