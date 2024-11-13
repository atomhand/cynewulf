use bevy::prelude::*;

pub mod indexes;

mod picking_backend;

pub mod fleet;
pub use fleet::Fleet;

pub mod galaxy_index;

pub mod selection;
pub use selection::Selection;

pub mod galaxy_config;
pub use galaxy_config::GalaxyConfig;

mod hypernet_pathfinding;
pub use hypernet_pathfinding::Pathfinding;

pub mod navigation_filter;

pub mod description;
pub use description::Description;

mod star;
pub use star::{Star,OverlaysTriangulationVertex};

mod hypernet;
pub use hypernet::Hypernet;

pub mod colony;
pub use colony::{StarClaim,Colony};

pub mod empire;
pub use empire::Empire;

mod planet;
pub use planet::Planet;
pub struct GalaxySetupPlugin;

impl Plugin for GalaxySetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((selection::SelectionPlugin, picking_backend::PickingBackendPlugin))
            .insert_resource(GalaxyConfig::default())
            .insert_resource(galaxy_index::GalaxyIndex::default())
            .insert_resource(Hypernet::new())
            .insert_resource(empire::PlayerEmpire { empire : None })
            //.insert_resource(SelectedObject{hovered_star : None})
            .add_systems(Update, (description::update_descriptions_system));
    }
}