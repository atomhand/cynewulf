use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
};

pub struct StarBillboardPlugin;

impl Plugin for StarBillboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MaterialPlugin::<PlanetBillboardMaterial>::default(),));
    }
}

// PLANET - SYSTEM VIEW BILLBOARD

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PlanetBillboardMaterial {
    #[uniform(0)]
    pub surface_color: Vec3,
    #[uniform(1)]
    pub halo_color: Vec3,
    #[uniform(2)]
    pub planet_radius: f32,
    #[uniform(3)]
    pub star_pos: Vec3,
    alpha_mode: AlphaMode,
}
impl PlanetBillboardMaterial {
    pub fn new(color: Vec3, star_pos: Vec3, radius: f32) -> Self {
        Self {
            surface_color: color,
            halo_color: Vec3::splat(1.0),
            planet_radius: radius,
            star_pos: star_pos,
            alpha_mode: AlphaMode::Add,
        }
    }
}

impl Material for PlanetBillboardMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/shader_planet_closeup.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/shader_planet_closeup.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}
