use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef}
};

pub struct StarBillboardPlugin;

impl Plugin for StarBillboardPlugin {
    fn build(&self, app : &mut App) {
        app.add_plugins( (
            MaterialPlugin::<PlanetBillboardMaterial>::default(),
            MaterialPlugin::<StarBillboardMaterial>::default(),
            MaterialPlugin::<SystemStarBillboardMaterial>::default()
        ));
    }
}

// PLANET - SYSTEM VIEW BILLBOARD

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct PlanetBillboardMaterial {
    #[uniform(0)]
    pub surface_color : Vec3,
    #[uniform(1)]
    pub halo_color : Vec3,
    #[uniform(2)]
    pub planet_radius : f32,
    #[uniform(3)]
    pub star_pos : Vec3,
    alpha_mode: AlphaMode
}
impl PlanetBillboardMaterial {
    pub fn new(color : Vec3, star_pos : Vec3, radius : f32) -> Self {
        Self {
            surface_color : color,
            halo_color : Vec3::splat(1.0),
            planet_radius : radius,
            star_pos : star_pos,
            alpha_mode : AlphaMode::Add
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

// SYSTEM VIEW BILLBOARD

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct SystemStarBillboardMaterial {
    #[uniform(0)]
    pub color : Vec3,
    #[uniform(1)]
    pub star_radius : f32,
    alpha_mode: AlphaMode
}
impl SystemStarBillboardMaterial {
    pub fn new(color : Vec3, radius : f32) -> Self {
        Self {
            color,
            star_radius : radius,
            alpha_mode : AlphaMode::Add
        }
    }
}

impl Material for SystemStarBillboardMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/shader_star_closeup.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/shader_star_closeup.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

// GALAXY VIEW BILLBOARD

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct StarBillboardMaterial {
    #[uniform(0)]
    pub color : Vec3,
    #[uniform(1)]
    pub star_radius : f32,
    #[uniform(2)]
    pub system_transition_factor : f32,
    alpha_mode: AlphaMode
}
impl StarBillboardMaterial {
    pub fn new(color : Vec3, radius : f32) -> Self {
        Self {
            color,
            star_radius : radius,
            system_transition_factor : 0.0,
            alpha_mode : AlphaMode::Add
        }
    }
}

impl Material for StarBillboardMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/shader_star_galaxy.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/shader_star_galaxy.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}