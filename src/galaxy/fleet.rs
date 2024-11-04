use bevy::prelude::*;
use crate::simulation::fleet_behaviour::navigation::*;
use crate::prelude::*;

#[derive(Component)]
pub struct Fleet {
    pub owner : Entity
}

#[derive(Component)]
pub struct FleetColonyCrew {
    pub colonists : i64,
    pub destination : Option<Entity>
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

use crate::camera::{CameraSettings,CameraMode,CameraMain};

pub fn fleet_preview_gizmos(
    nav_query : Query<(&NavPosition, &Fleet)>,
    empire_query : Query<&Empire>,
    hypernet : Res<Hypernet>,
    camera_settings : Res<CameraSettings>,
    camera : Query<&CameraMain>,
    mut gizmos : Gizmos
) {
    let cam = camera.get_single().unwrap();
    let transition = cam.adjusted_mode_transition();
    for (nav,fleet) in nav_query.iter() {
        let empire = empire_query.get(fleet.owner).unwrap();
        let galaxy = nav.galaxy_view_translation(&hypernet);
        let system = nav.system_view_translation(&hypernet);

        let scale = f32::lerp(1.0,GalaxyConfig::SOLAR_RADIUS, transition);
        gizmos.sphere(galaxy.lerp(system,transition), Quat::IDENTITY, scale, empire.color);
    }
}