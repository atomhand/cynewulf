use bevy::prelude::*;
use crate::galaxy::Planet;

pub fn update_orbiters(mut query: Query<(&mut Planet,&mut Transform)>) {
    for (mut planet, mut transform) in &mut query {
        planet.orbital_date = planet.orbital_date+1;
        if planet.orbital_date == planet.orbital_period { planet.orbital_date = 0; }
        planet.update_position();
        transform.translation = planet.system_local_pos();
    }
}