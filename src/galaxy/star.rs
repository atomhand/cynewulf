use bevy::prelude::*;
use super::GalaxyConfig;

#[derive(Component)]
pub struct OverlaysTriangulationVertex {
    pub node_id : u32
}

#[derive(Component)]
pub struct Star {
    pub pos : Vec3,
    pub node_id : u32,
    pub orbiters : Vec::<Entity>, // includes self id at index [0]
    pub mass : f32, // in stellar masses
    pub name : String,
}

impl Star {
    fn system_radius_au(&self) -> f32 {
        7.0
    }
    pub fn system_radius_actual(&self) -> f32{
        self.system_radius_au() * GalaxyConfig::AU_SCALE
    }

    pub fn new(star_name_gen : &mut crate::generators::markov_chain::StarNameGenerator, id : u32, pos : Vec3, stellar_masses : f32) -> Star {
        Star {
            node_id : id,
            pos,
            orbiters : Vec::new(),
            mass : stellar_masses,
            name : star_name_gen.next()
        }
    }

    pub fn get_raw_radius(&self) -> f32 {
        // return as fraction of Sun mass
        self.mass.sqrt()
    }
    
    pub fn get_scaled_radius(&self) -> f32 {
        // return as fraction of Sun radius
        self.get_raw_radius() * GalaxyConfig::SOLAR_RADIUS
    }

    // https://physics.stackexchange.com/questions/6771/star-surface-temperature-vs-mass/6772#6772
    // Kelvin
    pub fn get_temperature(&self) -> f32 {
        self.mass.powf(0.625) * 5772.0
    }

    pub fn _get_luminosity(&self) -> f32 {
        self.mass.powf(3.5)
    }

    fn simple_planck(temperature : f32) -> Vec3 {
        let mut res : Vec3 = Vec3::ZERO;
        let m = 1.0;
        for i in 0..3 {  // +=.1 if you want to better sample the spectrum.
            let f = 1.+0.5*i as f32; 
            res[i as usize] += 10.0 / m * (f*f*f) / (f32::exp(19.0e3*f/temperature) - 1.);  // Planck law
        }

        //res = res / res.max_element();
        res
    }

    pub fn get_color(&self) -> Vec3 {
        let planck = Self::simple_planck(self.get_temperature());
        planck
    }
}