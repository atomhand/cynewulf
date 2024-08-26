mod markov;
mod star_name_generator;
mod planet_name_generator;

use markov::MarkovChainModel;

pub use star_name_generator::StarNameGenerator;
pub use planet_name_generator::{PlanetNameGenerator,UsedPlanetNames};