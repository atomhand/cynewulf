use super::hypernet_pathfinding::Pathfinding;
use crate::prelude::*;
use bevy::prelude::*;
use delaunator::{triangulate, Point, EMPTY};
use petgraph::prelude::*;
use rand::prelude::*;
use std::collections::HashSet;

#[derive(Clone)]
pub struct Hypernode {
    pub pos: Vec3,
    pub star: Option<StarHandle>,
}

impl Hypernode {
    fn new(pos: Vec3) -> Self {
        Self { pos, star: None }
    }
}

#[derive(Clone)]
pub struct Hyperlane {
    pub length: i32,
}

#[derive(Resource)]
pub struct Hypernet {
    pub graph: StableGraph<Hypernode, Hyperlane, Undirected, u32>,
}
impl Hypernet {
    pub fn new() -> Self {
        Self {
            graph: StableGraph::<_, _, Undirected>::default(),
        }
    }

    pub fn num_lanes(&self) -> i32 {
        self.graph.edge_count() as i32
    }
    pub fn star(&self, id: u32) -> StarHandle {
        self.graph.node_weight(id.into()).unwrap().star.unwrap()
    }

    pub fn build_from_points(
        &mut self,
        points: &Vec<Point>,
        length_remove_threshold: f32,
        removal_rate: f32,
    ) {
        self.import(points);
        self.remove_over_length(length_remove_threshold);
        self.remove_random((self.graph.edge_count() as f32 * removal_rate) as u32, 12);

        // clear out the dead nodes
        // lazy method but it's not a hot loop
        let temp = Graph::from(self.graph.clone());
        self.graph = StableGraph::<Hypernode, Hyperlane, Undirected, u32>::from(temp);
    }

    fn import(&mut self, points: &Vec<Point>) {
        let del = triangulate(points);

        for point in points {
            self.graph.add_node(Hypernode::new(Vec3::new(
                point.x as f32,
                0.0,
                point.y as f32,
            )));
        }

        for i in 0..del.halfedges.len() {
            let a = del.triangles[i] as u32;
            let hb = del.halfedges[i];

            if hb != EMPTY {
                let b = del.triangles[hb] as u32;
                if a < b {
                    let a_pos = self.graph.node_weight(a.into()).unwrap().pos;
                    let b_pos = self.graph.node_weight(b.into()).unwrap().pos;
                    self.graph.add_edge(
                        a.into(),
                        b.into(),
                        Hyperlane {
                            length: (GalaxyConfig::GALACTIC_INTEGER_SCALE as f32
                                * a_pos.distance(b_pos)) as i32,
                        },
                    );
                }
            }
        }

        // Mark nodes on and adjacent to the hull, and remove all their edges
        // (We keep the nodes themselves - but not as stars - because they're used for triangulating the territory overlays)
        let mut to_clear: HashSet<usize> = std::collections::HashSet::new();
        for i in del.hull {
            for b in self.graph.neighbors(NodeIndex::new(i)) {
                to_clear.insert(b.index());
            }
            to_clear.insert(i);
        }
        self.graph.retain_edges(|g, e| {
            let (a, b) = g.edge_endpoints(e).unwrap();
            !(to_clear.contains(&a.index()) || to_clear.contains(&b.index()))
        });
    }

    fn get_distance_without_link(&self, a: u32, b: u32) -> Option<usize> {
        let path = self.find_path_without_direct_edge(a, b);

        if let Some(path) = path {
            Some(path.edges.len())
        } else {
            None
        }
    }

    fn remove_over_length(&mut self, length_factor: f32) {
        let mut total_length: i32 = 0;
        let mut num = 0;

        for e in self
            .graph
            .edge_indices()
            .map(|x| self.graph.edge_weight(x).unwrap())
        {
            total_length += e.length;
            num += 1;
        }

        let length = total_length as f64 / (num as f64) * length_factor as f64;
        let sqlen = (length * length) as f32;

        for e in self.graph.edge_indices().collect::<Vec<_>>() {
            let (a, b) = self.graph.edge_endpoints(e).unwrap();

            let an = self.graph.node_weight(a).unwrap();
            let bn = self.graph.node_weight(b).unwrap();
            let d = an.pos.distance_squared(bn.pos);

            if d > sqlen && self.get_distance_without_link(a.index() as u32, b.index() as u32).is_some()
            {
                self.graph.remove_edge(e);
            }
        }
    }

    // Remove a number of random edges without splitting the graph
    // at each iteration, the removed edge cannot result in a new distance between its vertices > max_new_distance
    // The distance restriction isn't perfect (future iterations can & will increase the distance between previously split vertices) but it's better than nothing
    fn remove_random(&mut self, number: u32, max_new_distance: u32) {
        let mut candidate_edges = self.graph.edge_indices().collect::<Vec<_>>();

        let mut i = 0;
        let mut rng = rand::rng();
        while i < number {
            if candidate_edges.is_empty() {
                break;
            }

            let r = rng.random_range(0..candidate_edges.len());

            let e = candidate_edges[r];
            candidate_edges.swap_remove(r);

            let (a, b) = self.graph.edge_endpoints(e).unwrap();
            if let Some(dist) = self.get_distance_without_link(a.index() as u32, b.index() as u32) {
                if dist < max_new_distance as usize {
                    self.graph.remove_edge(e);
                    i += 1;
                }
            }
        }
    }
}
