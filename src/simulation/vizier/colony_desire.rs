use bevy::prelude::*;
use crate::prelude::*;

use bevy::ecs::entity::EntityHashMap;

// SIMPLE COLONY DESIRE SCHEME

// COLONY RATING SCORE
// Steps
//   Base Rating is 10 000 000
//   1. Unreachable system rating is set to 0
//   2. SUBTRACT distance to the Capital
//   3. SUBTRACT distance to nearest owned system
//   4. If there are no owned planets in the system, multiply the score by EXPANSION factor
//      Else, multiply the score by CONSOLIDATE factor

struct ColonyRating {
    valid : bool,
    distance : Option<i32>, // distance from nearest planet that can supply a colony ship

    security : Option<f32> // Vulnerability to attack
    // Treat it as a pathfinding problem -
    // Each attacker threatens a system based on a function of their power, their likelihood to attack, and their "distance" from their core bases to the system
    // Distance: edge weights account for both the actual distance (reduced weight), and your defensive forces in the system. Defensive forces of your own allies may also be accounted for.
    // -- Could also account for defensive forces of a neutral power who is assumed to never ally/grant military access to the potential attacker. (This is a relatively common situation -
    //  granting military access requires a lot of trust & is typicaly equivalent to alliance, because there will be no arbtrary mechanics to prevent declaring war while accessing an empire)


    // Special qualities

    // CHOKEPOINT / DEFENSIVE BASE
    // TRADE VALUE
    // MILITARY / OFFENSIVE BASE
    // -- Rated based on , probably similar to the inversion of the security calculation above

    // NOTES ON COLONY PLANNING

    // Basic principle: Integration with other planning mechanisms, where the empire's high-level needs flow through to more specific requirements,
    // terminating in a decision like "Create a colony at X position."

    // Example high-level requirements
    // -- "Improve economic growth"
    // -- "Acquire living space to take pressure off overpopulated planets"
    // -- "Secure against military threats"
    // -- "Project military power / prepare for military expansion"

    // Derived requirement
    // -- "Establish defensive bases at chokepoints"
    // -- "Establish major military facilities "

    // Observation
    // -- A derived requirement that potentially results in a "create colony" instruction is just as likely to be satisfied by expanding and improving an existing colony,
    // if an appropriate one is available
}

#[derive(Component)]
struct ColonyVizier {
    owned_planets : Vec<(Entity,u32)>,
    map : EntityHashMap<ColonyRating>
}

impl Default for ColonyVizier {
    fn default() -> Self {
        Self {
            owned_planets : Vec::new(),
            map : default()
        }
    }
}

fn rate_colonies_system(
    planet_query : Query<(Entity,&Planet,Option<&Colony>),Without<Empire>>,
    star_query : Query<(&Star,&StarClaim),Without<Empire>>,
    mut empire_query : Query<(&Empire,&mut ColonyVizier)>,
    hypernet : Res<Hypernet>
) {
    for (_empire, mut viz) in empire_query.iter_mut() {
        viz.owned_planets.clear();
    }

    for (entity,planet,colony) in &planet_query {
        if let Some(owner) = colony.and_then(|x| Some(x.owner)) {
            let (_empire,mut viz) = empire_query.get_mut(owner).unwrap();
            viz.owned_planets.push((entity,planet.star_id));
        }
    }


    empire_query.par_iter_mut().for_each(|(empire,mut vizier)| {
        // array of distances to stars (indexed by hypernet node id)
        // Currently the source points are any planet owned by the empire
        // This needs to be improved
        // - at a bare minimum, restricting the sources to planets which are specifically capable of building and filling a colony ship

        // really, you want a more complex reachability score, where a planet is 
        // This might require projecting multiple different dijkstra maps from different source points
        let dijkstra = hypernet.dijkstra(&vizier.owned_planets.iter().map(|x| x.1).collect::<Vec<_>>() );

        for (entity,planet,colony) in &planet_query {
            vizier.map.insert(entity, ColonyRating {
                distance : dijkstra[planet.star_id as usize],
                security : None,
                valid : colony.is_none()
            });
        }
    });
}