use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef}
};

pub struct UiMaterialsPlugin;

impl Plugin for UiMaterialsPlugin {
    fn build(&self, app : &mut App) {
        app.add_plugins(UiMaterialPlugin::<StarIconMaterial>::default());
    }
}


#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct StarIconMaterial {
    #[uniform(0)]
    pub color : Vec4,
    //#[uniform(1)]
    //pub radius : f32,
}

// All functions on `UiMaterial` have default impls. You only need to implement the
// functions that are relevant for your material.
impl UiMaterial for StarIconMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/star_icon.wgsl".into()
    }
}