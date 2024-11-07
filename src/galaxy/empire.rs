use bevy::prelude::*;
use rand::prelude::*;
use super::navigation_filter::NavigationMask;
use crate::prelude::*;

use crate::generators::markov_chain::{UsedPlanetNames,PlanetNameGenerator};

#[derive(Component)]
pub struct Empire {
    pub color : Color,
    pub name : String,
    pub namegen : PlanetNameGenerator
}

#[derive(Bundle)]
pub struct EmpireBundle {
    empire : Empire,
    nav_mask : NavigationMask
}

impl Empire {
    pub fn random(rng : &mut ThreadRng, hypernet : &Hypernet, used_planet_names : &mut UsedPlanetNames) -> EmpireBundle {
        let mut namegen = PlanetNameGenerator::new(used_planet_names);


        EmpireBundle {
            empire : Self {
                color : Color::srgb(rng.gen(),rng.gen(),rng.gen()),
                name : namegen.next(used_planet_names),
                namegen : namegen,
            },
            nav_mask : NavigationMask::new(hypernet)
        }
    }
}


#[derive(Resource)]
pub struct PlayerEmpire {
    pub empire : Option<Entity>
}