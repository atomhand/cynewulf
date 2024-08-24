use bevy::prelude::*;
use super::navigation::*;
use crate::prelude::*;

#[derive(Component)]
pub struct Fleet {
    pub owner : Entity
}

#[derive(Component,Default)]
pub struct SystemFleetInfo {
    pub fleets : Vec<Entity>
}

#[derive(Bundle)]
pub struct FleetBundle {
    fleet : Fleet,
    nav_position : NavPosition,
    navigator : Navigator
}

impl FleetBundle {
    pub fn new(owner : Entity, system : u32) -> Self {
        Self {
            fleet : Fleet {
                owner
            },
            nav_position : NavPosition {
                root_system : system,
                offset : NavOffset::Star(Vec3::new(1.0,0.0,0.0) * GalaxyConfig::AU_SCALE)
            },
            navigator : Navigator {
                plan_queue : Vec::new(),
                action : Action::Idle,
                speed : GalaxyConfig::AU_SCALE * 0.5,
                hyperspeed : 10000
            }
        }
    }
}

use crate::camera::{CameraSettings,CameraMode};

pub fn fleet_preview_gizmos(
    nav_query : Query<(&mut NavPosition, &Fleet)>,
    empire_query : Query<&Empire>,
    hypernet : Res<Hypernet>,
    camera_settings : Res<CameraSettings>,
    mut gizmos : Gizmos
) {
    match camera_settings.camera_mode {
        CameraMode::Galaxy => {
            for (nav,fleet) in nav_query.iter() {
                let empire = empire_query.get(fleet.owner).unwrap();
                gizmos.sphere(nav.galaxy_view_translation(&hypernet), Quat::IDENTITY, 1.0, empire.color);
            }
        },
        CameraMode::Star => {
            for (nav,fleet) in nav_query.iter() {
                let empire = empire_query.get(fleet.owner).unwrap();
                gizmos.sphere(nav.system_view_translation(&hypernet), Quat::IDENTITY, GalaxyConfig::SOLAR_RADIUS, empire.color);
            }
        }
    }
}