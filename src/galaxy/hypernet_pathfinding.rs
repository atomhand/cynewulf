
use bevy::prelude::*;
use crate::prelude::*;
use super::Hypernet;
use std::collections::BinaryHeap;

pub trait Pathfinding {
    fn find_path_without_direct_edge(&self, a : u32, b : u32) -> Option<Vec<u32>>;

    fn find_path(&self, a : u32, b : u32) -> Option<Vec<u32>>;
}

use std::cmp::Ordering;

#[derive(Eq)]
struct PathfindingNode {
    star : u32,
    parent : u32,
    origin_dist : i32,
    heuristic_val : i32
}
impl PathfindingNode {
    fn score(&self) -> i32 {
        0 - (self.origin_dist + self.heuristic_val)
    }
    fn new(star : u32, parent : u32, dist : i32, dest_pos : Vec3, hypernet : &Hypernet) -> Self {
        Self {
            star,
            parent,
            origin_dist : dist,
            heuristic_val : (hypernet.graph.node_weight(star.into()).unwrap().pos.distance(dest_pos)  * GalaxyConfig::GALACTIC_INTEGER_SCALE as f32) as i32
        }
    }
}
impl PartialEq for PathfindingNode {
    fn eq(&self, other : &Self) -> bool {
        self.score().eq(&other.score())
    }
}
impl PartialOrd for PathfindingNode {
    fn partial_cmp(&self, other : &Self) -> Option<Ordering> {
        self.score().partial_cmp(&other.score())
    }
}

impl Ord for PathfindingNode {
    fn cmp(&self, other : &Self) -> Ordering {
        self.score().cmp(&other.score())
    }
}

impl Pathfinding for super::Hypernet {
    fn find_path_without_direct_edge(&self, star_a : u32, star_b : u32,) -> Option<Vec::<u32>> {
        let (n,_) = self.graph.capacity();
        let mut parent_id = vec![None; n];
        let mut closed = vec![false; n];
        let mut open = BinaryHeap::new();
    
        let dest_pos = self.graph.node_weight(star_b.into()).unwrap().pos;
    
        open.push(PathfindingNode::new(star_a, star_a, 0, dest_pos, &self));
        closed[star_a as usize] = true;
        parent_id[star_a as usize] = Some(star_a);
    
        while let Some(top) = open.pop() {
            parent_id[top.star as usize] = Some(top.parent);
    
            if top.star == star_b {
                break;
            }
    
            for n in self.graph.neighbors(top.star.into()) {
                if top.star == star_a && n.index() as u32 == star_b { continue; }
                let e = self.graph.edge_weight(self.graph.find_edge(top.star.into(),n).unwrap()).unwrap();
                if !closed[n.index()] {
                    closed[n.index()] = true;
                    open.push(PathfindingNode::new(n.index() as u32, top.star, top.origin_dist + e.length,dest_pos, &self));
                }
            }
        }
    
        let mut path = Vec::new();
        let mut curr = star_b;
    
        loop {
            path.push(curr);
            if curr == star_a {
                path.reverse();
                return Some(path);
            }
            if let Some(c) = parent_id[curr as usize] {
                curr = c;
            } else {
                return None;
            }
        }
    }
    
    fn find_path(&self, star_a : u32, star_b: u32) -> Option<Vec::<u32>> {
        let (n,_) = self.graph.capacity();
        let mut parent_id = vec![None; n];
        let mut closed = vec![false; n];
        let mut open = BinaryHeap::new();
    
        let dest_pos = self.graph.node_weight(star_b.into()).unwrap().pos;
    
        open.push(PathfindingNode::new(star_a, star_a, 0, dest_pos, &self));
        closed[star_a as usize] = true;
        parent_id[star_a as usize] = Some(star_a);
    
        while let Some(top) = open.pop() {
            parent_id[top.star as usize] = Some(top.parent);
    
            if top.star == star_b {
                break;
            }
    
            for n in self.graph.neighbors(top.star.into()) {
                let e = self.graph.edge_weight(self.graph.find_edge(top.star.into(),n).unwrap()).unwrap();
                if !closed[n.index()] {
                    closed[n.index()] = true;
                    open.push(PathfindingNode::new(n.index() as u32, top.star, top.origin_dist + e.length,dest_pos, &self));
                }
            }
        }
    
        let mut path = Vec::new();
        let mut curr = star_b;
    
        loop {
            path.push(curr);
            if curr == star_a {
                path.reverse();
                return Some(path);
            }
            if let Some(c) = parent_id[curr as usize] {
                curr = c;
            } else {
                return None;
            }
        }
    }
}