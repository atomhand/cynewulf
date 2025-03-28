use crate::prelude::*;
use bevy::prelude::*;

use crate::galaxy::selection::{GalaxySelectable, SystemSelectable};
use rand::prelude::*;

use crate::galaxy::Description;

use delaunator::Point;

use crate::galaxy::OverlaysTriangulationVertex;
use std::f32::consts::PI;

pub fn setup_stars(
    mut commands: Commands,
    galaxy_config: Res<GalaxyConfig>,
    mut hypernet: ResMut<Hypernet>,
    mut galaxy_index: ResMut<GalaxyIndex>,
) {
    let mut rng = rand::rng();
    let mut points: Vec<Point> = Vec::with_capacity(galaxy_config.max_stars as usize);
    let min_sqd = galaxy_config.spacing * galaxy_config.spacing;

    // Place new star randomly in the galactic circle
    for _i in 0..galaxy_config.max_stars {
        // TODO: Account for galactic star density function
        // -- SImple way: (try first) Reservoir sampling -- Less simple: acceleration structure
        for _j in 0..100 {
            let r = galaxy_config.radius * rng.random::<f32>().sqrt();
            let theta = 2.0 * PI * rng.random::<f32>();
            let point = Point {
                x: (r * theta.cos()) as f64,
                y: (r * theta.sin()) as f64,
            };

            let mut has_clearance = true;
            for other in &points {
                let sq_d = ((point.x - other.x) * (point.x - other.x)
                    + (point.y - other.y) * (point.y - other.y)) as f32;
                if sq_d < min_sqd {
                    has_clearance = false;
                    break;
                }
            }

            if has_clearance {
                points.push(point);
                break;
            }
        }
    }

    hypernet.build_from_points(&points, 1.5, 0.6);

    let mut starname_gen = super::markov_chain::StarNameGenerator::new();

    for node_id in hypernet.graph.node_indices().collect::<Vec<_>>() {
        let node = hypernet.graph.node_weight(node_id).unwrap();
        let star_pos = node.pos; //Vec3::new(node.pos.x as f32,0.0,node.pos.y as f32);

        // The hypernet generation has detected nodes on or adjacent to the hull of the pointset & removed all of their connections
        // These nodes don't spawn stars, but we still need them in order to provide vertices for the triangulation this is used for map overlays rendering
        // .. It would make a lot of sense to just build and cache the triangulation at this step, and save the trouble of dropping this stuff into the ECS
        let is_enabled = hypernet.graph.edges(node_id).count() > 0;

        if is_enabled {
            let mut star = Star::new(
                &mut starname_gen,
                node_id.index() as u32,
                star_pos,
                Star::random_star_mass(&mut rng),
            );

            let rad = star.get_scaled_radius();
            let starname = star.name.clone();

            let num_planets = rng.random_range(0..8);
            let mut planets: Vec<Entity> = Vec::new();

            for i in 0..num_planets {
                let planet = Planet::make_random(&star);
                let rad = planet.get_visual_radius();
                let planet_identifier = char::from_u32(i + 98).unwrap();
                planets.push(
                    commands
                        .spawn((
                            planet,
                            SystemSelectable {
                                radius: rad + 4.0 * GalaxyConfig::SOLAR_RADIUS,
                            },
                            Description::planet(format!("{} {}", starname, planet_identifier)),
                        ))
                        .id(),
                );
            }

            let parent = commands
                .spawn((
                    StarClaim {
                        claimed_tick: 0,
                        owner: None,
                    },
                    SystemIndex::default(),
                    OverlaysTriangulationVertex {
                        node_id: node_id.index() as u32,
                    },
                    SystemSelectable { radius: rad * 1.75 },
                    GalaxySelectable { radius: 10.0 },
                    Description::star(starname),
                    crate::galaxy::fleet::SystemFleetInfo::default(),
                    Transform::from_translation(star_pos),
                    Visibility::Inherited,
                ))
                .id();

            let star_handle = galaxy_index.register_star(parent, node_id.index());
            commands.entity(parent).insert(star_handle);

            star.orbiters.push(parent);
            star.orbiters.extend_from_slice(planets.as_slice());

            hypernet.graph.node_weight_mut(node_id).unwrap().star = Some(star_handle);

            commands.entity(parent).insert(star).add_children(&planets);

            for p in planets {
                let planet_handle = galaxy_index.register_planet(star_handle, p);
                commands.entity(p).insert(planet_handle);
            }
        } else {
            commands.spawn((
                OverlaysTriangulationVertex {
                    node_id: node_id.index() as u32,
                },
                Transform::from_translation(star_pos),
            ));
        }
    }
}
