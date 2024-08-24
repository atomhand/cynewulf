use super::MarkovChainModel;
use std::collections::HashSet;
pub struct StarNameGenerator {
    markov : MarkovChainModel,
    used_names : HashSet<String>
}

impl StarNameGenerator {
    pub fn new() -> Self {
        let mut markov = MarkovChainModel::new(3);
        let mut used_names : HashSet<String> = HashSet::new();
        let mut names = Vec::<String>::new();
        for starname in crate::galaxy::Star::NAMES {
            names.push(starname.to_string());
            used_names.insert(starname.to_string());
        }
        markov.build(&names, 0.00001);

        Self {
            markov,
            used_names
        }
    }

    pub fn next(&mut self) -> String {
        let mut res : String = self.markov.generate();

        while res.len() > 15 || self.used_names.contains(&res) {
            res = self.markov.generate();
        }
        self.used_names.insert(res.clone());

        res
    }
}