mod empires_index;
pub mod galaxy_index;
mod system_index;

use bevy::prelude::*;
use crate::prelude::*;

pub use empires_index::EmpireIndex;
pub use system_index::SystemIndex;

pub struct IndexPlugin;

impl Plugin for IndexPlugin {
    fn build(&self, app : &mut App) {
        app.add_systems(SimPostTick, (empires_index::update_empire_index_system, system_index::update_system_index_system));
    }
}