use super::indexes::EmpireIndex;
use super::navigation_filter::NavigationMask;
use crate::prelude::*;
use bevy::prelude::*;
use rand::prelude::*;

use crate::generators::markov_chain::{PlanetNameGenerator, UsedPlanetNames};

#[derive(Component)]
pub struct Empire {
    pub color: Color,
    pub name: String,
    pub namegen: PlanetNameGenerator,
}

#[derive(Bundle)]
pub struct EmpireBundle {
    empire: Empire,
    nav_mask: NavigationMask,
    empire_index: EmpireIndex,
}

impl Empire {
    pub fn random(
        rng: &mut ThreadRng,
        hypernet: &Hypernet,
        used_planet_names: &mut UsedPlanetNames,
    ) -> EmpireBundle {
        let mut namegen = PlanetNameGenerator::new(used_planet_names);

        EmpireBundle {
            empire: Self {
                color: Color::srgb(rng.random(), rng.random(), rng.random()),
                name: namegen.next(used_planet_names),
                namegen,
            },
            nav_mask: NavigationMask::new(hypernet, true),
            empire_index: default(),
        }
    }
}

#[derive(Resource)]
pub struct PlayerEmpire {
    pub empire: Option<Entity>,
}
