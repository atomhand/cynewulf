use bevy::{input::mouse::MouseWheel, prelude::*};

use super::galaxy::Selection;
use crate::galaxy::GalaxyConfig;
use crate::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(PostUpdate, camera_control_system)
            .insert_resource(CameraSettings::default());
    }
}

fn spawn_camera(mut commands: Commands, mut clearcolor: ResMut<ClearColor>) {
    *clearcolor = ClearColor(Color::BLACK);
    commands.spawn((
        // NEED TO SET CLEAR COLOR TO BLACK...
        Camera3d { ..default() },
        Transform::from_xyz(10.0, 12.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraMain::default(),
    ));
}

#[derive(PartialEq)]
pub enum CameraMode {
    Star,
    Galaxy,
}

#[derive(Component, Clone)]
pub struct CameraMain {
    target_pos: Vec3,
    star_pos: Vec3,
    star_local_pos: Vec3,
    system_radius: f32,
    zoom: f32,
    smooth_zoom_buffer: f32,
    dragging: Option<Vec3>,
    pub mode_transition: f32,
}

impl Default for CameraMain {
    fn default() -> Self {
        Self {
            target_pos: Vec3::new(0.0, 0., 0.0),
            zoom: 1.0,
            system_radius: 1.0,
            star_local_pos: Vec3::ZERO,
            smooth_zoom_buffer: 0.0,
            star_pos: Vec3::ZERO,
            dragging: None,
            mode_transition: 0.0,
        }
    }
}

impl CameraMain {
    pub fn adjusted_mode_transition(&self) -> f32 {
        smootherstep(0.0, 1.0, self.mode_transition)
    }

    fn translation(&self, transition: f32, galaxy_scale: f32) -> Vec3 {
        let adjusted_system_scale = self.system_radius * 2.0;

        let galaxy_zoom = self.zoom * 0.85 + 0.15;
        let adjusted_galaxy_scale = galaxy_scale * galaxy_zoom;

        let adjusted_scale =
            adjusted_galaxy_scale * (1.0 - transition) + adjusted_system_scale * transition;

        let antitilt = 0.6;
        self.look_pos(transition) + Vec3::new(0., adjusted_scale, -adjusted_scale * antitilt)
    }

    fn look_pos(&self, transition: f32) -> Vec3 {
        let system_scale = self.system_radius * 2.0;
        let star_mode_look_pos =
            self.star_pos + self.star_local_pos + Vec3::new(0.0, 0.0, -0.1 * system_scale); // This is a fudged offset to make the star system better centred on camera.. there has to be a better way. :)
        self.target_pos * (1.0 - transition) + transition * star_mode_look_pos
    }
}

use ::bevy::render::extract_component::ExtractComponent;
use bevy::ecs::query::QueryItem;
impl ExtractComponent for CameraMain {
    type QueryData = &'static CameraMain;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_, Self::QueryData>) -> Option<Self> {
        Some(item.clone())
    }
}

#[derive(Resource)]
pub struct CameraSettings {
    pub star: Option<Entity>,
    pub camera_mode: CameraMode,
    pub visibility_updated: bool,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            camera_mode: CameraMode::Galaxy,
            star: None,
            visibility_updated: false,
        }
    }
}

use bevy::input::mouse::MouseScrollUnit;

fn smootherstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let x = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);

    x * x * x * (x * (6.0 * x - 15.0) + 10.0)
}

pub fn camera_control_system(
    mut query: Query<(&Camera, &mut Transform, &mut CameraMain)>,
    star_query: Query<&Star>,
    windows: Query<&Window>,
    mut camera_settings: ResMut<CameraSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    galaxy_config: Res<GalaxyConfig>,
    mut selection: ResMut<Selection>,
    mut scroll_evr: EventReader<MouseWheel>,
    //mut gizmos : Gizmos,
) {
    let galaxy_scale = galaxy_config.radius * 2.5;
    let (cam, mut transform, mut camera_main) =
        query.get_single_mut().expect("Error: Require ONE camera");

    // HIDE CURSOR
    //windows.single_mut().cursor.visible = false;

    let Ok(window) = windows.get_single() else {
        return;
    };

    let cursor = window.cursor_position(); // cache this cause we will use it twice
    let mouse_world_pos = cursor
        .and_then(|cursor| {
            cam.viewport_to_world(&GlobalTransform::from(*transform), cursor)
                .ok()
        })
        .and_then(|ray| {
            ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
                .map(|distance| ray.get_point(distance))
        });

    let mut key_delta = Vec3::ZERO;

    if mouse_buttons.pressed(MouseButton::Middle) {
        if camera_main.dragging.is_none() {
            camera_main.dragging = mouse_world_pos;
        }
    } else {
        camera_main.dragging = None;

        // no keyboard movement unless drag pan is not active
        if keys.pressed(KeyCode::KeyW) {
            key_delta.z += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            key_delta.x += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            key_delta.z -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            key_delta.x -= 1.0;
        }
    }

    match camera_settings.camera_mode {
        CameraMode::Star => {
            let mut wheel_ev: f32 = 0.0;
            for ev in scroll_evr.read() {
                wheel_ev = wheel_ev.min(ev.y);
            }
            if wheel_ev < 0.0 || selection.zoomed_system.is_none() {
                camera_settings.camera_mode = CameraMode::Galaxy;
                camera_settings.visibility_updated = false;
                camera_main.target_pos = camera_main.star_pos;
                camera_settings.star = None;
                camera_main.zoom = 0.0;
                selection.zoomed_system = None;
                // reset camera drag to avoid weird stuff
                camera_main.dragging = None;
            }
        }
        CameraMode::Galaxy => {
            if let Some(star_ent) = selection.zoomed_system {
                camera_settings.camera_mode = CameraMode::Star;
                camera_settings.visibility_updated = false;
                camera_main.star_local_pos = Vec3::ZERO;
                let star = star_query.get(star_ent).unwrap();
                camera_main.star_pos = star.pos;
                camera_main.system_radius = star.system_radius_actual();
                camera_settings.star = Some(star_ent);
                // reset camera drag to avoid weird stuff
                camera_main.dragging = None;
            }
        }
    }

    let transition_speed = 4.0 * time.delta_secs();

    let old_zoom = camera_main.zoom;

    // Update
    match camera_settings.camera_mode {
        CameraMode::Star => {
             // wipe smooth zoom buffer so nothing weird happens when switching modes
             // In future there is probably going to be zoom in star view, so wipe should happen specifically when transitioning
            camera_main.smooth_zoom_buffer = 0.0;

            let speed: f32 = GalaxyConfig::AU_SCALE * 6.0 * time.delta_secs();
            camera_main.star_local_pos += key_delta * speed;

            if camera_main.star_local_pos.length() > camera_main.system_radius {
                camera_main.star_local_pos =
                    camera_main.star_local_pos.normalize() * camera_main.system_radius;
            }
            camera_main.mode_transition = (camera_main.mode_transition + transition_speed).min(1.0);
        }
        CameraMode::Galaxy => {
            // scroll delta is cached to a buffer 
            // buffer is converted to actual zoom over time for a smooth zooming effect
            for ev in scroll_evr.read() {
                match ev.unit {
                    MouseScrollUnit::Line => {
                        //camera_main.zoom -= ev.y * 0.05;
                        camera_main.smooth_zoom_buffer += ev.y;
                    }
                    MouseScrollUnit::Pixel => {
                        //camera_main.zoom -= ev.y * 0.05;
                        camera_main.smooth_zoom_buffer += ev.y;
                    }
                }
            }

            let smooth_zoom_min = 0.001f32;
            let smooth_zoom_factor = 0.2f32;

            let smooth_zoom_amount = if camera_main.smooth_zoom_buffer < 0.0 {
                f32::min(camera_main.smooth_zoom_buffer*smooth_zoom_factor,(-smooth_zoom_min).max(camera_main.smooth_zoom_buffer))
            } else {
                f32::max(camera_main.smooth_zoom_buffer*smooth_zoom_factor,smooth_zoom_min.min(camera_main.smooth_zoom_buffer))
            };
            camera_main.zoom -= smooth_zoom_amount * 0.05;
            camera_main.smooth_zoom_buffer -= smooth_zoom_amount;

            camera_main.zoom = camera_main.zoom.clamp(0., 1.);
            let tzoom = camera_main.zoom * 0.85 + 0.15;
            let speed: f32 = (tzoom * galaxy_scale) * 0.5 * time.delta_secs();
            camera_main.target_pos += key_delta * speed;
            let d = camera_main.target_pos.xz().length();
            if d > galaxy_config.radius {
                camera_main.target_pos *= galaxy_config.radius / d;
            }
            camera_main.mode_transition = (camera_main.mode_transition - transition_speed).max(0.0);
        }
    }

    //
    if camera_main.zoom != old_zoom && camera_main.dragging.is_none() {
        camera_main.dragging = mouse_world_pos;
    }

    for _i in 0..2 {
        transform.translation =
            camera_main.translation(camera_main.adjusted_mode_transition(), galaxy_scale);
        transform.look_at(
            camera_main.look_pos(camera_main.adjusted_mode_transition()),
            Vec3::Y,
        );

        let Some(mouse_pos) = cursor
            .and_then(|cursor| {
                cam.viewport_to_world(&GlobalTransform::from(*transform), cursor)
                    .ok()
            })
            .and_then(|ray| {
                ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
                    .map(|distance| ray.get_point(distance))
            })
        else {
            return;
        };

        if let Some(drag_origin) = camera_main.dragging {
            let drag_offset = drag_origin - mouse_pos;

            camera_main.target_pos += drag_offset;
        }

        transform.translation =
            camera_main.translation(camera_main.adjusted_mode_transition(), galaxy_scale);
        transform.look_at(
            camera_main.look_pos(camera_main.adjusted_mode_transition()),
            Vec3::Y,
        );

        /* MOUSE LATENCY TESTING GIZMO
        if i == 1 {

            gizmos.sphere(mouse_pos, Quat::IDENTITY, 1.0, Color::linear_rgb(1.0,0.0,0.0));
        }
        */
    }
}
