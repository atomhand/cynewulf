use bevy::prelude::*;
use rand::prelude::*;

use crate::markov_chain::PlanetNameGenerator;

#[derive(Component)]
pub struct Empire {
    pub color : Color,
    pub namegen : PlanetNameGenerator
}

impl Empire {
    pub fn random(rng : &mut ThreadRng, used_planet_names : &mut crate::markov_chain::UsedPlanetNames) -> Self {
        Self {
            color : Color::srgb(rng.gen(),rng.gen(),rng.gen()),
            namegen : PlanetNameGenerator::new(used_planet_names)
        }
    }
}

#[derive(Component)]
pub struct PlayerEmpire;