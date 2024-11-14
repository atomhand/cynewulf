

use bevy::prelude::*;

mod instanced_star_pipeline;

mod draw_galaxy;
pub struct GraphicsPlugin;

mod galaxy_materials;
mod ui_materials;

pub use ui_materials::StarIconMaterial;

mod territory_overlay;

use draw_galaxy::draw_system_overlays;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ui_materials::UiMaterialsPlugin,galaxy_materials::StarBillboardPlugin, draw_galaxy::DrawGalaxyPlugin,territory_overlay::OverlaysPlugin))
            .add_systems(Update, draw_system_overlays);
    }
}