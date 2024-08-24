use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::galaxy::GalaxyConfig;
use super::galaxy::Selection;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app : &mut App) {
        app.add_systems(Startup,spawn_camera)
            .add_systems(Update,camera_control_system)
            .insert_resource(CameraSettings::default());
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(
        (Camera3dBundle {
            camera : Camera {
                clear_color : ClearColorConfig::Custom(Color::srgb(0.0,0.0,0.0)),
                ..default()
            },
            transform: Transform::from_xyz(10.0,12.0,16.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraMain::default())
    );
}

#[derive(PartialEq)]
pub enum CameraMode {
    Star,
    Galaxy
}

#[derive(Component,Clone)]
pub struct CameraMain {
    target_pos: Vec3,
    star_pos : Vec3,
    star_local_pos: Vec3,
    zoom: f32,
    pub mode_transition: f32
}

impl Default for CameraMain {
    fn default() -> Self {
        Self {
            target_pos: Vec3::new(0.0, 0., 0.0),
            zoom: 1.0,
            star_local_pos : Vec3::ZERO,
            star_pos : Vec3::ZERO,
            mode_transition: 0.0,
        }
    }
}

impl CameraMain {
    pub fn adjusted_mode_transition(&self) -> f32 {
        smootherstep(0.0,1.0,self.mode_transition)
    }
}

use::bevy::render::extract_component::ExtractComponent;
use bevy::ecs::query::QueryItem;
impl ExtractComponent for CameraMain {
    type QueryData = &'static CameraMain;
    type QueryFilter = ();
    type Out = Self;

    fn extract_component(item: QueryItem<'_,Self::QueryData>) -> Option<Self> {
        Some(item.clone())
    }
}

#[derive(Resource)]
pub struct CameraSettings {
    pub star : Option<Entity>,
    pub camera_mode: CameraMode,
    pub visibility_updated : bool,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            camera_mode: CameraMode::Galaxy,
            star : None,
            visibility_updated : false
        }
    }
}

use bevy::input::mouse::MouseScrollUnit;

fn smootherstep(edge0 : f32, edge1 : f32, x : f32) -> f32 {
    let x = ((x-edge0) / (edge1-edge0)).clamp(0.0,1.0);

    x * x * x * (x * (6.0 * x -15.0) + 10.0)
}

pub fn camera_control_system(
    mut query: Query<(&mut Transform, &mut CameraMain)>,
    transform_query: Query<&Transform, Without<CameraMain>>,
    mut camera_settings : ResMut<CameraSettings>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    galaxy_config : Res<GalaxyConfig>,
    mut selection : ResMut<Selection>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    let galaxy_scale = galaxy_config.radius * 2.5;

    let (mut transform, mut camera) = query.get_single_mut().expect("Error: Require ONE camera");

    // TODO - BLENDED TRANSITIONS BETWEEN CAMERA MODES
    // ie. have an anim s value
    // Calculate positions for "galaxy camera" and "star camera"
    // Lerp
    //
    // Input/camera movement only calculated for active mode
    //
    // In the future, need a system for smoothly deciding when to start drawing planets etc.
    // Efficient 
    match camera_settings.camera_mode {
        CameraMode::Star => {            
            let mut wheel_ev : f32 = 0.0;
            for ev in scroll_evr.read() {
                wheel_ev = wheel_ev.min(ev.y);
            }
            if wheel_ev < 0.0 || selection.zoomed_system == None {
                camera_settings.camera_mode = CameraMode::Galaxy;
                camera_settings.visibility_updated = false;
                camera.target_pos = camera.star_pos;
                camera_settings.star = None;
                camera.zoom = 0.0;
                selection.zoomed_system = None;
            }
        },
        CameraMode::Galaxy => {
            if let Some(star) = selection.zoomed_system {
                camera_settings.camera_mode = CameraMode::Star;
                camera_settings.visibility_updated = false;
                //controller.zoom = 0.0;
                camera.star_local_pos = Vec3::ZERO;
                camera.star_pos = transform_query.get(star).unwrap().translation;
                camera_settings.star = Some(star);
            }
        }
    }

    let mut key_delta = Vec3::ZERO;

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

    // 

    let transition_speed = 4.0 * time.delta_seconds();

    // Update 
    match camera_settings.camera_mode {
        CameraMode::Star => {
            let speed: f32 = GalaxyConfig::AU_SCALE * 6.0 * time.delta_seconds();
     
            // TODO: CLAMP TO SYSTEM DIAMETER
            camera.star_local_pos += key_delta * speed;

            // Adjust mode transition
            camera.mode_transition = (camera.mode_transition  + transition_speed).min(1.0);

            // star selection gfraphic
        },
        CameraMode::Galaxy => {
            for ev in scroll_evr.read() {
                match ev.unit {
                    MouseScrollUnit::Line => {
                        camera.zoom -= ev.y * 0.05;
                    }
                    MouseScrollUnit::Pixel => {
                        camera.zoom -= ev.y * 0.05;
                    }
                }
            }        
        
            camera.zoom = camera.zoom.clamp(0., 1.);
            let tzoom = camera.zoom * 0.85 + 0.15;
        
            let speed: f32 = (tzoom* galaxy_scale) * 0.5 * time.delta_seconds();
        
            let world_bounds_max = Vec2::splat(galaxy_config.radius);   

            let world_bounds_min = -world_bounds_max;   
            camera.target_pos += key_delta * speed;
            camera.target_pos.x = camera.target_pos.x.clamp(world_bounds_min.x, world_bounds_max.x);
            camera.target_pos.z = camera.target_pos.z.clamp(world_bounds_min.y, world_bounds_max.y);

            /*  
            transform.translation = controller.target_pos + Vec3::new(0., scale,-scale * (1.0-tzoom * 0.75)) * tzoom;
            transform.look_at(controller.target_pos, Vec3::Y);
            */
            // Adjust mode transition

            camera.mode_transition = (camera.mode_transition - transition_speed).max(0.0);

        }
    }
    
    // Apply blended camera position + rotation with mode transition

    let adjusted_mode_transition = camera.adjusted_mode_transition();

    let star_mode_look_pos = camera.star_pos + camera.star_local_pos;
    let look_pos = camera.target_pos * (1.0 - adjusted_mode_transition) + adjusted_mode_transition * star_mode_look_pos;
    
    let galaxy_zoom =camera.zoom * 0.85 + 0.15;
    let adjusted_galaxy_scale = galaxy_scale * galaxy_zoom;

    let adjusted_system_scale = GalaxyConfig::AU_SCALE * 16.0;

    let adjusted_scale = adjusted_galaxy_scale * (1.0 - adjusted_mode_transition) + adjusted_system_scale * adjusted_mode_transition;

    //transform.translation = look_pos + Vec3::new(0., galaxy_scale,-galaxy_scale * (1.0-azoom * 0.75)) * azoom;
    let antitilt = 0.6;
    transform.translation = look_pos + Vec3::new(0., adjusted_scale, -adjusted_scale * antitilt);
    transform.look_at(look_pos, Vec3::Y);
}
