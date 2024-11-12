use bevy::prelude::*;
use crate::simulation::fleet_behaviour::navigation::*;
use crate::prelude::*;

#[derive(Component)]
pub struct Fleet {
    pub owner : Entity,
    pub time_since_last_jump : u32
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
    pub fn new(owner : Entity, local_offset : Vec3, system : u32) -> Self {
        Self {
            fleet : Fleet {
                time_since_last_jump : 0,
                owner
            },
            nav_position : NavPosition {
                root_system : system,
                offset : NavOffset::Star(local_offset)
            },
            navigator : Navigator {
                plan_queue : Vec::new(),
                action : Action::Idle,
                stranded_go_home : false,
                speed : GalaxyConfig::AU_SCALE * 0.5,
                hyperspeed : 10000
            }
        }
    }
}

use crate::camera::{CameraSettings,CameraMode,CameraMain};

pub fn fleet_preview_gizmos(
    nav_query : Query<(&NavPosition,&Navigator, &Fleet)>,
    empire_query : Query<&Empire>,
    hypernet : Res<Hypernet>,
    camera_settings : Res<CameraSettings>,
    camera : Query<&CameraMain>,
    mut gizmos : Gizmos
) {
    let cam = camera.get_single().unwrap();
    let transition = cam.adjusted_mode_transition();
    for (nav_pos,navigator,fleet) in nav_query.iter() {
        let empire = empire_query.get(fleet.owner).unwrap();
        let galaxy = nav_pos.galaxy_view_translation(&hypernet);
        let system = nav_pos.system_view_translation(&hypernet);

        let scale = f32::lerp(1.0,GalaxyConfig::SOLAR_RADIUS, transition);
        //gizmos.sphere(galaxy.lerp(system,transition), Quat::IDENTITY, scale, empire.color);
        gizmos.circle(galaxy.lerp(system,transition), Dir3::Y, scale * 0.75, empire.color);

        let indicator_col = match navigator.action {
            Action::Idle => Color::linear_rgb(0.25,0.25,0.25),
            Action::Move(_) => Color::linear_rgb(0.0,1.0,0.0),
            Action::Colonise(_) => Color::linear_rgb(0.0,0.0,1.0),
            _ => Color::NONE
        };

        gizmos.circle(galaxy.lerp(system,transition), Dir3::Y, scale * 1.5, indicator_col);
    }
}