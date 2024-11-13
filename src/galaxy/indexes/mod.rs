mod empires_index;
pub mod galaxy_index;

use bevy::prelude::*;
use crate::prelude::*;

pub use empires_index::EmpireIndex;

pub struct IndexPlugin;

impl Plugin for IndexPlugin {
    fn build(&self, app : &mut App) {
        app.add_systems(SimPostTick, empires_index::update_empire_index_system);
    }
}