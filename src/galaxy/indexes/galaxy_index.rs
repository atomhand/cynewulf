use crate::prelude::*;
use bevy::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct StarHandle {
    index: u32,
    pub entity: Entity,
}

#[derive(Component, Clone, Copy)]
pub struct PlanetHandle {
    index: u32,
    pub entity: Entity,
}

#[derive(Resource)]
pub struct GalaxyIndex {
    planets: Vec<(Entity, Option<u32>)>,
    stars: Vec<(Entity, Option<u32>)>,
}

pub struct PlanetsIterator<'a> {
    galaxy_index: &'a GalaxyIndex,
    next: Option<u32>,
}

impl<'a> Iterator for PlanetsIterator<'a> {
    type Item = PlanetHandle;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(planet_id) = self.next {
            let (planet_entity, next) = self.galaxy_index.planets[planet_id as usize];
            self.next = next;
            Some(PlanetHandle {
                index: planet_id,
                entity: planet_entity,
            })
        } else {
            None
        }
    }
}

impl Default for GalaxyIndex {
    fn default() -> Self {
        GalaxyIndex {
            planets: default(),
            stars: default(),
        }
    }
}

impl GalaxyIndex {
    pub fn register_star(&mut self, entity: Entity, id: usize) -> StarHandle {
        self.stars.resize(
            usize::max(self.stars.len(), id + 1),
            (Entity::PLACEHOLDER, None),
        );
        self.stars[id] = (entity, None);
        StarHandle {
            index: id as u32,
            entity,
        }
    }

    pub fn register_planet(&mut self, parent: StarHandle, entity: Entity) -> PlanetHandle {
        let (star, pointer) = self.stars[parent.index as usize];
        let id = self.planets.len() as u32;
        self.planets.push((entity, pointer));
        self.stars[parent.index as usize] = (star, Some(id));

        PlanetHandle { index: id, entity }
    }

    pub fn get_orbiters(&self, star: StarHandle) -> PlanetsIterator {
        let (_, pointer) = self.stars[star.index as usize];
        PlanetsIterator {
            galaxy_index: self,
            next: pointer,
        }
    }
}
