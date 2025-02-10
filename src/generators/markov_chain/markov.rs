use std::collections::{HashMap,HashSet};

use rand::{rng, prelude::*};

// Primary reference - https://www.roguebasin.com/index.php?title=Names_from_a_high_order_Markov_Process_and_a_simplified_Katz_back-off_scheme

struct ObservedCount {
    counts : HashMap::<char,f32>,
    total : f32,
}

impl ObservedCount {

    fn new(support : &HashSet::<char>, prior : f32) -> Self {

        let mut counts = HashMap::<char,f32>::new();
        for c in support {
            // base count for all outputs is "prior" (this is the cover against sitations with no prior data)
            counts.insert(*c,prior);
        }

        let total = counts.len() as f32 * prior;

        Self {
            counts,
            total
        }
    }

    fn observe(&mut self, event : char) {
        self.counts.insert(event, self.counts[&event] + 1.0 );
        self.total += 1.0;
    }

    fn sample(&self) -> char {
        // sample a char from counts, weighted by the associated count/weight
        let r = rng().random_range(0.0..self.total);

        let mut running_count = 0.0;
        for (char,weight) in self.counts.iter() {
            running_count += weight;
            if running_count > r {
                return *char;
            }
        }
        '#' // error symbol
    }
}

pub struct MarkovChainModel {
    counts : HashMap::<String,ObservedCount>,
    support : HashSet<char>,
    order : usize,
}

impl MarkovChainModel {
    const STARTCHAR : char = '(';
    const ENDCHAR : char = ')';

    pub fn new(order : usize) -> Self {
        Self {
            counts : HashMap::new(),
            support : HashSet::new(),
            order
        }
    }
    fn observe(&mut self, word : &String, prior : f32) {
        let chars : Vec::<char> = word.chars().collect();
        let mut sequence = Vec::<char>::with_capacity(chars.len() + 1 + self.order);
        for _i in 0..self.order {
            sequence.push(Self::STARTCHAR);
        }
        for char in word.chars() {
            sequence.push(char);
        }
        sequence.push(Self::ENDCHAR);

        for i in self.order..sequence.len() {
            let context = &sequence[(i-self.order)..i];
            let event = sequence[i];

            for j in 0..context.len() {
                let subcontext : String = context[j..context.len()].into_iter().collect();
                if !self.counts.contains_key(&subcontext) {
                    self.counts.insert(subcontext.clone(), ObservedCount::new(&self.support,prior));
                }

                let count = self.counts.get_mut(&subcontext).unwrap();

                count.observe(event);
            }
        }
    }

    fn backoff(&self, context : &[char]) -> String {
        let mut context : Vec<char> = context.to_vec();

        if context.len() > self.order {
            context = context[(context.len()-self.order)..].to_vec();
            assert_eq!(context.len(),self.order);
        }
        while context.len() < self.order {
            context.insert(0,Self::STARTCHAR);
        }

        while !self.counts.contains_key(&context.iter().collect::<String>()) && context.len() > 0 {
            context.remove(0);
        }

        context.iter().collect()
    }

    fn sample(&self, seq : &Vec<char>) -> Option<char> {
        let context = self.backoff(seq);

        return  self.counts.get(&context).and_then(|x| Some(x.sample()));
    }
    
    pub fn generate(&self) -> String {
        return self.generate_iter(0);
    }

    fn generate_iter(&self, iter : i32) -> String {
        let mut seq : Vec<char> = Vec::new();

        if iter >= 1000 {
            return "FAILED_NAME_GENERATION".to_string();
        }

        while seq.len() == 0 || seq[seq.len()-1] != Self::ENDCHAR {
            if let Some(next_sample) = self.sample(&seq) {
                seq.push(next_sample);
            } else {
                return self.generate_iter(iter+1);
            }
        }

        return seq[..seq.len()-1].iter().collect();
    }

    // default for prior 0.01?

    pub fn build(&mut self, inputs : &Vec::<String>, prior : f32) {
        for word in inputs {
            for ch in word.chars() {
                self.support.insert(ch);
            }
        }
        self.support.insert(Self::ENDCHAR);

        for word in inputs {
            self.observe(word, prior);
        }
    }
}