use super::galaxy_materials::{GalaxyVolumeMaterial, PlanetBillboardMaterial};
use crate::camera::{CameraMode, CameraSettings};
use crate::galaxy::Selection;
use crate::prelude::*;
use bevy::prelude::*;

pub struct DrawGalaxyPlugin;

impl Plugin for DrawGalaxyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(super::instanced_star_pipeline::StarMaterialPlugin)
            .add_systems(
                BuildGalaxyGraphics,
                (finish_assemble_star_system, star_gfx, place_galaxy_volume),
            )
            .add_systems(Update, update_planet_materials);
    }
}

fn place_galaxy_volume(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut galaxy_materials: ResMut<Assets<GalaxyVolumeMaterial>>,
    galaxy_config: Res<GalaxyConfig>,
) {
    let galaxy_mesh = meshes.add(Cuboid::from_size(Vec3::splat(2.0)));
    let mat = galaxy_materials.add(GalaxyVolumeMaterial::new(galaxy_config.radius));
    commands.spawn((
        Mesh3d(galaxy_mesh),
        Transform::IDENTITY,
        Visibility::Inherited,
        MeshMaterial3d(mat),
        bevy::render::view::NoFrustumCulling,
    ));
}

fn finish_assemble_star_system(
    planets: Query<(&Planet, Entity), Added<Planet>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut planet_materials: ResMut<Assets<PlanetBillboardMaterial>>,
) {
    for (planet, entity) in &planets {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Rectangle::from_size(Vec2::splat(2.0)))),
            MeshMaterial3d(planet_materials.add(PlanetBillboardMaterial::new(
                Vec3::new(1.0, 0.5, 0.0),
                planet.star_pos,
                planet.get_visual_radius(),
            ))),
            Visibility::Inherited,
            Transform::from_translation(planet.system_local_pos()),
        ));
    }
}

pub fn update_planet_materials(
    planet_query: Query<(&MeshMaterial3d<PlanetBillboardMaterial>, Option<&Colony>)>,
    star_query: Query<&Star>,
    empire_query: Query<&Empire>,
    mut planet_materials: ResMut<Assets<PlanetBillboardMaterial>>,
    selection: Res<Selection>,
) {
    let Some(star_ent) = selection.zoomed_system else {
        return;
    };
    let Ok(star) = star_query.get(star_ent) else {
        return;
    };

    for orbiter in &star.orbiters {
        let Ok((planet, colony)) = planet_query.get(*orbiter) else {
            continue;
        };
        let Some(mat) = planet_materials.get_mut(planet) else {
            continue;
        };

        let empire_col = colony
            .and_then(|x| empire_query.get(x.owner).ok())
            .map(|x| x.color)
            .unwrap_or(Color::srgb(0.6, 0.6, 0.6));

        mat.halo_color = if Some(*orbiter) == selection.hovered {
            if Some(*orbiter) == selection.selected {
                Color::WHITE
            } else {
                Color::srgb(1.0, 0.0, 0.0)
            }
        } else if Some(*orbiter) == selection.selected {
            Color::WHITE
        } else {
            empire_col
        }
        .to_srgba()
        .to_vec3();
    }
}

fn star_gfx(
    stars: Query<(&Star, Entity), Added<Star>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let billboardmesh = meshes.add(Rectangle::from_size(Vec2::splat(2.0)));
    commands.spawn((
        Mesh3d(billboardmesh.clone()),
        Transform::IDENTITY,
        Visibility::Inherited,
        super::instanced_star_pipeline::StarInstanceMaterialData(
            stars
                .iter()
                .map(
                    |(star, _)| super::instanced_star_pipeline::StarInstanceData {
                        position: star.pos,
                        star_radius: star.get_scaled_radius(),
                        color: Srgba::from_vec3(star.get_color()).to_f32_array(),
                    },
                )
                .collect(),
        ),
        bevy::render::view::NoFrustumCulling,
    ));
}

pub fn draw_system_overlays(stars: Query<&Star>, cam: Res<CameraSettings>, mut gizmos: Gizmos) {
    if cam.camera_mode == CameraMode::Star {
        if let Some(star_id) = cam.star {
            if let Ok(star) = stars.get(star_id) {
                gizmos.circle(
                    Isometry3d::new(
                        Vec3::new(star.pos.x, 0.0, star.pos.z),
                        Quat::from_rotation_x(std::f32::consts::PI / 2.0),
                    ),
                    star.system_radius_actual(),
                    Color::WHITE,
                );
            }
        }
    }
}
