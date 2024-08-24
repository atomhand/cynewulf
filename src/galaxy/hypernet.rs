
use bevy::prelude::*;
use delaunator::{Point,triangulate, EMPTY};
use rand::prelude::*;
use petgraph::prelude::*;
use crate::prelude::*;
use super::pathfinding::Pathfinding;

#[derive(Clone)]
pub struct HypernetConnection {
    pub dest : u32,
    pub length : i32
}

#[derive(Clone)]
pub struct Hypernode {
    pub pos : Vec3,
    pub star : Entity
    //pub connections : Vec::<HypernetConnection>,
}

impl Hypernode {
    fn new(pos : Vec3) -> Self {
        Self {
            pos,
            star : Entity::PLACEHOLDER
            //connections : Vec::new(),
        }
    }
    /*
    fn add(&mut self, other : u32, length : i32) {
        self.connections.push(HypernetConnection{ dest: other, length});
    }
    fn remove(&mut self, other : u32) {
        for i in 0..self.connections.len() {
            if self.connections[i].dest == other {
                self.connections.swap_remove(i);
                break;
            }
        }
    }

    fn clear(&mut self) {
        self.connections.clear();
    }
    */
}

#[derive(Clone)]
pub struct Hyperlane {
    //pub node_a : u32,
    //pub node_b : u32,
    pub length : i32,
}

use std::collections::HashSet;

#[derive(Resource)]
pub struct Hypernet {
    pub graph : StableGraph::<Hypernode,Hyperlane, Undirected, u32>,
    //pub nodes : Vec::<Hypernode>,
    //pub lanes : Vec::<Hyperlane>,
}
impl Hypernet {
    /*
    fn remove_link(&mut self, a : u32, b : u32) {
        self.nodes[a as usize].remove(b);
        self.nodes[b as usize].remove(a);
    }
    */

    pub fn new() -> Self {
        Self {
            graph : StableGraph::<_,_,Undirected>::default(),
            //nodes : Vec::new(),
            //lanes : Vec::new(),
            //stars_index : Vec::new()
        }
    }

    pub fn num_lanes(&self) -> i32 {
        self.graph.edge_count() as i32
    }

    pub fn build_from_points(&mut self, points : &Vec<Point>, length_remove_threshold : f32, removal_rate : f32) {
        self.import(points);
        self.remove_over_length(length_remove_threshold);

        // this is just to gen an initial count
        //self.finalise_links();

        self.remove_random((self.graph.edge_count() as f32 * removal_rate) as u32, 12);
        //self.finalise_links();
    }

    fn import(&mut self, points : &Vec<Point>) {
        let del = triangulate(&points);



        for point in points {
            self.graph.add_node(Hypernode::new(Vec3::new(point.x as f32,0.0,point.y as f32)));
            /*
            self.nodes.push(
                Hypernode::new(Vec3::new(point.x as f32,0.0,point.y as f32))
            )
            */
        }

        for i in 0..del.halfedges.len() {
            let a = del.triangles[i] as u32;
            let hb = del.halfedges[i];
            
            if hb != EMPTY {
                let b = del.triangles[hb] as u32;
                if  a < b {
                    /*
                    let dist = (10000. * Vec3::distance(self.nodes[a].pos,self.nodes[b].pos)) as i32;
                    self.nodes[a].add(b as u32, dist);
                    self.nodes[b].add(a as u32, dist);
                    */
                    self.graph.add_edge(a.into(),b.into(), Hyperlane {
                        //node_a : a as u32,
                        //node_b : b as u32,
                        length : (GalaxyConfig::GALACTIC_INTEGER_SCALE as f32 * Vec3::distance(self.graph.node_weight(a.into()).unwrap().pos, self.graph.node_weight(b.into()).unwrap().pos)) as i32
                    });
                }
            }
        }

        // Remove nodes on the hull
        // + for nodes adjacent to 
        let mut to_clear : HashSet<usize> = std::collections::HashSet::new();
        for i in del.hull {
            for b in self.graph.neighbors(NodeIndex::new(i)) {
                to_clear.insert(b.index());
            }
            to_clear.insert(i);
        }
        self.graph.retain_edges(|g,e|{
            let (a,b) = g.edge_endpoints(e).unwrap();
            !(to_clear.contains(&a.index()) || to_clear.contains(&b.index()))
        });
    }

    fn get_distance_without_link(&self, a : u32, b : u32) -> Option<usize> {
        let path = self.find_path_without_direct_edge(a,b);

        if let Some(path) = path {
            Some(path.len())
        } else {
            None
        }
    }

    fn remove_over_length(&mut self, length_factor : f32) {

        let mut total_length : i32 = 0;
        let mut num = 0;

        for e in self.graph.edge_indices().map(|x| self.graph.edge_weight(x).unwrap()) {
            total_length += e.length;
            num += 1;
        }

        let length = total_length as f64 / (num as f64) * length_factor as f64;
        let sqlen = (length*length) as f32;

        for e in self.graph.edge_indices().collect::<Vec<_>>() {
            let (a,b) = self.graph.edge_endpoints(e).unwrap();

            let an = self.graph.node_weight(a).unwrap();
            let bn = self.graph.node_weight(b).unwrap();
            let d = an.pos.distance_squared(bn.pos);

            if d > sqlen {
                if let Some(_) = self.get_distance_without_link(a.index() as u32,b.index() as u32) {
                    self.graph.remove_edge(e);
                }
            }
        }
    }

    // Remove a number of random edges without splitting the graph
    // at each iteration, the removed edge cannot result in a new distance between its vertices > max_new_distance
    // The distance restriction isn't perfect (future iterations can & will increase the distance between previously split vertices) but it's better than nothing
    fn remove_random(&mut self, number : u32, max_new_distance : u32) {

        let mut i = 0;

        let mut candidate_edges = self.graph.edge_indices().collect::<Vec<_>>();

        while i < number {
            if candidate_edges.len() == 0 { break; }

            let mut rng = rand::thread_rng();
            let r = rng.gen_range(0..candidate_edges.len());

            let e = candidate_edges[r];
            candidate_edges.swap_remove(r);

            let (a,b) = self.graph.edge_endpoints(e).unwrap();
            if let Some(dist) = self.get_distance_without_link(a.index() as u32, b.index() as u32) {
                if dist < max_new_distance as usize {
                    self.graph.remove_edge(e);
                    i += 1;
                }
            }
        }

        println!("removed {} links (target was {}).", i, number);
    }
}