use bevy::prelude::*;

mod empires_placement;
pub mod markov_chain;

pub mod galaxy_generation;

pub struct GalaxyGenerationPlugin;

impl Plugin for GalaxyGenerationPlugin {
    fn build(&self, app : &mut App) {
        app
            .insert_resource(markov_chain::UsedPlanetNames::default())
            .add_systems(Startup, (galaxy_generation::setup_stars,empires_placement::place_star_empires.after(galaxy_generation::setup_stars)))
            .add_systems(Update,empires_placement::finish_create_colony);
    }
}