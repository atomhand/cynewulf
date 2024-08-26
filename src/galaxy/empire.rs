use bevy::prelude::*;
use rand::prelude::*;

use crate::generators::markov_chain::{UsedPlanetNames,PlanetNameGenerator};

#[derive(Component)]
pub struct Empire {
    pub color : Color,
    pub name : String,
    pub namegen : PlanetNameGenerator
}

impl Empire {
    pub fn random(rng : &mut ThreadRng, used_planet_names : &mut UsedPlanetNames) -> Self {
        let mut namegen = PlanetNameGenerator::new(used_planet_names);
        Self {
            color : Color::srgb(rng.gen(),rng.gen(),rng.gen()),
            name : namegen.next(used_planet_names),
            namegen : namegen,
        }
    }
}

#[derive(Resource)]
pub struct PlayerEmpire {
    pub empire : Option<Entity>
}