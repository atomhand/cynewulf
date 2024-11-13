
use bevy::prelude::*;
use crate::prelude::*;
use crate::camera::{CameraSettings,CameraMode};
use crate::galaxy::Selection;
use super::galaxy_materials::PlanetBillboardMaterial;


pub struct DrawGalaxyPlugin;

impl Plugin for DrawGalaxyPlugin {
    fn build(&self, app : &mut App) {
        app.add_plugins(super::instanced_star_pipeline::StarMaterialPlugin)
            .add_systems(BuildGalaxyGraphics,(finish_assemble_star_system,star_gfx))
            .add_systems(Update, update_planet_materials);
    }
}

fn finish_assemble_star_system(
    planets : Query<(&Planet,Entity),Added<Planet>>,
    mut commands : Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut planet_materials: ResMut<Assets<PlanetBillboardMaterial>>,
) {
    for (planet,entity) in &planets {
        commands.entity(entity).insert(
            MaterialMeshBundle {
                mesh :  meshes.add(Rectangle::from_size(Vec2::splat(2.0))),
                material: planet_materials.add(PlanetBillboardMaterial::new(Vec3::new(1.0,0.5,0.0), planet.star_pos, planet.get_visual_radius())),
                visibility : Visibility::Inherited,
                transform : Transform::from_translation(planet.system_local_pos()),
                ..default()
            }
        );
    }
}

pub fn update_planet_materials(
    planet_query : Query<(&Handle<PlanetBillboardMaterial>,Option<&Colony>)>,
    star_query : Query<&Star, Without<Handle<PlanetBillboardMaterial>>>,
    empire_query : Query<&Empire, Without<Handle<PlanetBillboardMaterial>>>,
    mut planet_materials : ResMut<Assets<PlanetBillboardMaterial>>,
    selection : Res<Selection>
) {
    let Some(star_ent) = selection.zoomed_system else { return; };
    let Ok(star) = star_query.get(star_ent) else { return; };

    for orbiter in &star.orbiters {
        let Ok((planet,colony)) = planet_query.get(*orbiter) else { continue; };
        let Some(mat) = planet_materials.get_mut(planet) else { continue; };

        let empire_col = colony
            .and_then(|x| empire_query.get(x.owner).ok())
            .and_then(|x| Some(x.color)).unwrap_or(Color::srgb(0.6,0.6,0.6));
        
        mat.halo_color =  if Some(*orbiter) == selection.hovered {
            if Some(*orbiter) == selection.selected {
                Color::WHITE
            } else {
                Color::srgb(1.0,0.0,0.0)
            }
        } else if Some(*orbiter) == selection.selected {
            Color::WHITE
        } else {
            empire_col
        }.to_srgba().to_vec3();
    }
}

fn star_gfx(
    stars : Query<(&Star,Entity),Added<Star>>,
    mut commands : Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let billboardmesh = meshes.add(Rectangle::from_size(Vec2::splat(2.0)));
    commands.spawn((
        billboardmesh.clone(),
        SpatialBundle::INHERITED_IDENTITY,
        super::instanced_star_pipeline::StarInstanceMaterialData(
            stars.iter().map(|(star,_)|
            super::instanced_star_pipeline::StarInstanceData {
                position: star.pos,
                star_radius: star.get_scaled_radius(),
                color: Srgba::from_vec3(star.get_color()).to_f32_array(),
            }).collect(),
        ),
        bevy::render::view::NoFrustumCulling
    ));
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