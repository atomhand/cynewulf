use super::selection::{Selection, SystemSelectable};
use crate::prelude::*;
use crate::simulation::fleet_behaviour::navigation::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Fleet {
    pub owner: Entity,
    pub time_since_last_jump: u32,
}

#[derive(Component)]
pub struct FleetColonyCrew {
    pub colonists: i64,
    pub destination: Option<Entity>,
}

#[derive(Component, Default)]
pub struct SystemFleetInfo {
    pub fleets: Vec<Entity>,
}

#[derive(Bundle)]
pub struct FleetBundle {
    fleet: Fleet,
    nav_position: NavPosition,
    navigator: Navigator,
    selectable: SystemSelectable,
}

impl FleetBundle {
    pub fn new(owner: Entity, local_offset: Vec3, system: u32) -> Self {
        Self {
            fleet: Fleet {
                time_since_last_jump: 0,
                owner,
            },
            nav_position: NavPosition {
                root_system: system,
                offset: NavOffset::Star(local_offset),
            },
            navigator: Navigator {
                plan_queue: Vec::new(),
                action: Action::Idle,
                stranded_go_home: false,
                speed: GalaxyConfig::AU_SCALE * 0.5,
                hyperspeed: 10000,
            },
            selectable: SystemSelectable {
                radius: GalaxyConfig::SOLAR_RADIUS * 5.0,
            },
        }
    }
}

use crate::camera::CameraMain;

pub fn fleet_preview_gizmos(
    nav_query: Query<(Entity, &NavPosition, &Fleet)>,
    empire_query: Query<&Empire>,
    hypernet: Res<Hypernet>,
    selection: Res<Selection>,
    camera: Query<&CameraMain>,
    mut gizmos: Gizmos,
) {
    let cam = camera.get_single().unwrap();
    let transition = cam.adjusted_mode_transition();
    for (entity, nav_pos, fleet) in nav_query.iter() {
        let empire = empire_query.get(fleet.owner).unwrap();
        let galaxy = nav_pos.galaxy_view_translation(&hypernet);
        let system = nav_pos.system_view_translation(&hypernet);

        let scale = f32::lerp(1.0, GalaxyConfig::SOLAR_RADIUS, transition);

        let isometry = Isometry3d::new(
            galaxy.lerp(system, transition),
            Quat::from_rotation_x(std::f32::consts::PI / 2.0),
        );

        gizmos.circle(isometry, scale * 0.8, empire.color);

        let col = selection
            .get_selection_state(entity)
            .as_colour_with_default(Color::NONE);

        gizmos.circle(isometry, scale * 1.5, col);
    }
}
