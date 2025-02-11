use bevy::prelude::*;

#[derive(Resource)]
pub struct GalaxyConfig {
    pub radius: f32,
    pub max_stars: i32,
    pub spacing: f32,
}
impl GalaxyConfig {
    pub const AU_SCALE: f32 = 0.1;
    pub const CELESTIAL_BODIES_SCALE: f32 = 20.0; // boost to the radius of celestial bodies relative to distance
    pub const PLANETS_SCALE: f32 = 1.0; // radius of a jupiter-sized planet relative to a sun-size star
    pub const SOLAR_RADIUS: f32 = 0.00465 * Self::AU_SCALE * Self::CELESTIAL_BODIES_SCALE; // Radius of Sol
                                                                                           // === 0.0093
    pub const MAX_SYSTEM_BODIES: usize = 12; // Used for UI slots and stuff

    pub const HYPERLANE_VISUAL_STAR_CLEARANCE: f32 = 10.0;

    pub const GALACTIC_INTEGER_SCALE: i32 = 10000;
}
impl Default for GalaxyConfig {
    fn default() -> Self {
        Self {
            radius: 500.0,
            max_stars: 1000,
            spacing: 40.0,
            // Scale of 1 AU to a parsec
            // This doesn't need to be scaled realistically, it's just chosen to create a pleasant zoom transition from Galaxy-System view,
            // and to allow the use of a shared coordinate system for rendering
        }
    }
}
