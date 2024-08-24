
use bevy::prelude::*;
use crate::prelude::*;
use crate::camera_control::{CameraSettings,CameraMode,camera_control_system};

#[derive(Component)]
struct GalaxyViewComponent

;#[derive(Component)]
struct SystemViewComponent;

use super::galaxy_materials::{PlanetBillboardMaterial,StarBillboardMaterial,SystemStarBillboardMaterial};

fn finish_assemble_star_system(
    stars : Query<(&Star,Entity),Added<Star>>,
    planets : Query<(&Planet,Entity),Added<Planet>>,
    mut commands : Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut planet_materials: ResMut<Assets<PlanetBillboardMaterial>>,
    mut galaxy_materials: ResMut<Assets<StarBillboardMaterial>>,
    mut system_materials: ResMut<Assets<SystemStarBillboardMaterial>>,
) {

    let billboardmesh = meshes.add(Rectangle::from_size(Vec2::splat(2.0)));

    for (planet,entity) in &planets {
        commands.entity(entity).insert(
            MaterialMeshBundle {
                mesh : billboardmesh.clone(),
                material: planet_materials.add(PlanetBillboardMaterial::new(Vec3::new(1.0,0.5,0.0), planet.star_pos, planet.get_visual_radius())),
                visibility : Visibility::Inherited,
                transform : Transform::from_translation(planet.system_local_pos()),
                ..default()
            }
        );
    }

    /*
    for (star,entity) in &stars {

        let galaxy_star = commands.spawn(
            (MaterialMeshBundle {
                mesh : billboardmesh.clone(),
                material: galaxy_materials.add(StarBillboardMaterial::new(star.get_color(), star.get_scaled_radius())),
                visibility : Visibility::Visible,
                ..default()
            },GalaxyViewComponent)).id();
        let system_star = commands.spawn(
            (MaterialMeshBundle {
                mesh : billboardmesh.clone(),
                material: system_materials.add(SystemStarBillboardMaterial::new(star.get_color(), star.get_scaled_radius())),
                visibility : Visibility::Inherited,
                ..default()
            },SystemViewComponent)).id();
        commands.entity(entity).push_children(&[galaxy_star,system_star]);
    }
    */
    
    commands.spawn((
        billboardmesh.clone(),
        SpatialBundle::INHERITED_IDENTITY,
        super::instanced_star_material::InstanceMaterialData(
            stars.iter().map(|(star,_)|
            super::instanced_star_material::InstanceData {
                position: star.pos,
                star_radius: star.get_scaled_radius(),
                color: Srgba::from_vec3(star.get_color()).to_f32_array(),
            }).collect(),
        ),
        bevy::render::view::NoFrustumCulling
    ));
    
}

pub struct DrawGalaxyPlugin;

impl Plugin for DrawGalaxyPlugin {
    fn build(&self, app : &mut App) {
        app.add_plugins(super::instanced_star_material::StarMaterialPlugin)
            .add_systems(Update, finish_assemble_star_system);
    }
}

fn update_stars_zoom(
    cam_query: Query<&crate::camera_control::CameraMain>,
    star_mat_handles : Query<&Handle<StarBillboardMaterial>,With<GalaxyViewComponent>>,
    mut mats : ResMut<Assets<StarBillboardMaterial>>,
 ) {
    return;
    let cam = cam_query.get_single().unwrap();
    for mat_handle in star_mat_handles.iter() {
        let Some(mat) = mats.get_mut(mat_handle) else {continue;};
        mat.system_transition_factor = cam.mode_transition;
    }
 }

fn update_stars_visibility(
    mut galaxy_stars : Query<&mut Visibility, With<GalaxyViewComponent>>,
    mut system_stars : Query<&mut Visibility, (With<Star>, Without<GalaxyViewComponent>)>,
    mut cam : ResMut<CameraSettings>,
) {
    if !cam.visibility_updated {
        match cam.camera_mode {
            CameraMode::Galaxy => {
                for mut star_vis in galaxy_stars.iter_mut() {
                    *star_vis = Visibility::Hidden;
                }
                for mut star_vis in system_stars.iter_mut() {
                    *star_vis = Visibility::Hidden;
                }
            },
            CameraMode::Star => {
                for mut star_vis in galaxy_stars.iter_mut() {
                    //*star_vis = Visibility::Hidden;
                    *star_vis = Visibility::Hidden;
                }
                for mut star_vis in system_stars.iter_mut() {
                    //*star_vis = Visibility::Visible;
                    *star_vis = Visibility::Hidden;
                }
            }
        }

        cam.visibility_updated = true;
    }
}

use crate::galaxy::Selection;

pub fn draw_hyperlanes(
    hypernet : Res<Hypernet>,
    selection : Res<Selection>,
    stars_query : Query<&Star>,
    mut gizmos : Gizmos
) {
    if let Some(hovered) = selection.hovered {
        if let Some(selected_system) = selection.selected_system {
            if let Ok(star_b) = stars_query.get(hovered) {
                if let Ok(star_a) = stars_query.get(selected_system) {
                    if let Some(path) = hypernet.find_path(star_a.node_id, star_b.node_id) {
                        for star_p in path {
                            let p_pos = hypernet.graph.node_weight(star_p.into()).unwrap().pos;

                            gizmos.circle(p_pos, Dir3::Y, 8.0, Color::srgb(1.0,0.0,0.0));
                        }
                    }
                }
            }
        }
    }

    for edge in hypernet.graph.edge_indices() {
        let (an,bn) = hypernet.graph.edge_endpoints(edge).unwrap();
        let aw = hypernet.graph.node_weight(an).unwrap();
        let bw = hypernet.graph.node_weight(bn).unwrap();

        let dir = (bw.pos-aw.pos).normalize();

        let a = aw.pos + dir * GalaxyConfig::HYPERLANE_VISUAL_STAR_CLEARANCE;
        let b = bw.pos - dir * GalaxyConfig::HYPERLANE_VISUAL_STAR_CLEARANCE;

        gizmos.line(a,b, Color::srgb(155./255.,205./255.,0.));
    }
}

pub fn draw_system_overlays(
    stars : Query<&Star>,
    cam : Res<CameraSettings>,
    mut gizmos : Gizmos,
) {
    if cam.camera_mode == CameraMode::Star {
        if let Some(star_id) = cam.star {
            if let Ok(star) = stars.get(star_id) {
                gizmos.circle(star.pos, Dir3::Y, star.system_radius_actual(), Color::WHITE);
            }
        }
    }
}