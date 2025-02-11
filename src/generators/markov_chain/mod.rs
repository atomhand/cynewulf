mod markov;
mod planet_name_generator;
mod star_name_generator;

use markov::MarkovChainModel;

pub use planet_name_generator::{PlanetNameGenerator, UsedPlanetNames};
pub use star_name_generator::StarNameGenerator;
