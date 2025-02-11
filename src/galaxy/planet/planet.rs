use crate::prelude::*;
use bevy::prelude::*;
use rand::prelude::*;

#[derive(Component)]
pub struct Planet {
    au_scaled_pos: Vec3, // Planet pos is stored in AU, for convenience and accuracy. Needs to be rescaled for rendering etc.
    pub star_id: u32,    // for convenient access
    pub star_pos: Vec3,
    // In AU
    pub orbit_radius: f32,
    // in Earth days
    pub orbital_period: u32,
    pub orbital_date: u32,
    pub radius: f32, // in AU
    pub insolation: f32,
    // TO ADD
    // orbital_period
}

use std::f32::consts::PI;

impl Planet {
    pub fn get_visual_radius(&self) -> f32 {
        self.radius * GalaxyConfig::PLANETS_SCALE * GalaxyConfig::SOLAR_RADIUS // baseline planet size is Jupiter (~10% of the sun)
    }

    // Returns in millions of square km
    pub fn get_surface_area(&self) -> f32 {
        // rad of Jupiter = 70 million km
        4. * std::f32::consts::PI * (self.radius * 70.) * (self.radius * 70.)
    }

    pub fn get_population_support(&self) -> u64 {
        let insolation_penalty = f32::min(
            100.0,
            f32::max(
                self.insolation * self.insolation,
                1.0 / (self.insolation * self.insolation),
            ),
        );

        let earth_ref_capacity = 12 * 1000000000_u64;
        let earth_surface_area = 510;
        self.get_surface_area() as u64
            * (((earth_ref_capacity / earth_surface_area) * 100)
                / (insolation_penalty * 100.0) as u64)
    }

    // return pos rescaled to the general coordinate system
    pub fn system_local_pos(&self) -> Vec3 {
        self.au_scaled_pos * GalaxyConfig::AU_SCALE
    }

    pub fn make_random(star: &Star) -> Planet {
        let mut rng = rand::rng();
        let orbit_rad = rng.random_range(1.0..3.0);
        let period = (rng.random_range(2.0..3.0) * orbit_rad * 200.0) as u32;
        let orbital_date = rng.random_range(0..period);
        Self::new(
            star.pos,
            star.node_id,
            orbit_rad,
            period,
            orbital_date,
            rng.random_range(0.1..1.0),
            star.get_insolation(orbit_rad),
        )
    }

    pub fn new(
        star_pos: Vec3,
        star_id: u32,
        orbit_radius: f32,
        orbital_period: u32,
        orbital_date: u32,
        radius: f32,
        insolation: f32,
    ) -> Planet {
        let mut planet = Planet {
            au_scaled_pos: Vec3::ZERO,
            star_id,
            star_pos,
            orbit_radius,
            orbital_period,
            orbital_date,
            radius,
            insolation,
        };
        planet.update_position();
        planet
    }

    pub fn update_position(&mut self) {
        let angle = (self.orbital_date as f32 / self.orbital_period as f32) * 2.0 * PI;
        self.au_scaled_pos = Vec3::new(f32::cos(angle), 0.0, f32::sin(angle)) * self.orbit_radius;
    }
}
