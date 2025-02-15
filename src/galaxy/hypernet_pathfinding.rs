use super::Hypernet;
use crate::prelude::*;
use bevy::prelude::*;
use std::collections::BinaryHeap;

pub struct Path {
    pub nodes: Vec<u32>,
    pub edges: Vec<u32>,
}

impl Path {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn reverse(&mut self) {
        self.nodes.reverse();
        self.edges.reverse();
    }
}

pub trait Pathfinding {
    fn find_path_without_direct_edge(&self, a: u32, b: u32) -> Option<Path>;

    fn find_path_multi_source(&self, sources: &[u32], b: u32) -> Option<Path>;
    fn find_path(&self, a: u32, b: u32) -> Option<Path>;

    fn dijkstra(&self, input_points: &[u32]) -> Vec<Option<i32>>;
}

use std::cmp::Ordering;
#[derive(Eq)]
struct DijkstraNode {
    star: u32,
    cost: i32,
}
impl DijkstraNode {
    fn score(&self) -> i32 {
        -self.cost
    }
}
impl PartialEq for DijkstraNode {
    fn eq(&self, other: &Self) -> bool {
        self.score().eq(&other.score())
    }
}
impl PartialOrd for DijkstraNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DijkstraNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score().cmp(&other.score())
    }
}

#[derive(Eq)]
struct PathfindingNode {
    star: u32,
    parent: u32,
    edge_to_parent: u32,
    origin_dist: i32,
    heuristic_val: i32,
}
impl PathfindingNode {
    fn score(&self) -> i32 {
        0 - (self.origin_dist + self.heuristic_val)
    }
    fn new(star: u32, parent: &PathfindingNode, dest_pos: Vec3, hypernet: &Hypernet) -> Self {
        let edge_id = hypernet
            .graph
            .find_edge(star.into(), parent.star.into())
            .unwrap();
        let edge_w = hypernet.graph.edge_weight(edge_id).unwrap();
        let pos = hypernet.graph.node_weight(star.into()).unwrap().pos;
        Self {
            star,
            parent: parent.star,
            edge_to_parent: edge_id.index() as u32,
            origin_dist: parent.origin_dist + edge_w.length,
            heuristic_val: (pos.distance(dest_pos) * GalaxyConfig::GALACTIC_INTEGER_SCALE as f32)
                as i32,
        }
    }
}
impl PartialEq for PathfindingNode {
    fn eq(&self, other: &Self) -> bool {
        self.score().eq(&other.score())
    }
}
impl PartialOrd for PathfindingNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathfindingNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score().cmp(&other.score())
    }
}

impl Pathfinding for super::Hypernet {
    fn find_path_without_direct_edge(&self, star_a: u32, star_b: u32) -> Option<Path> {
        if star_a == star_b {
            return Some(Path {
                nodes: vec![star_a],
                edges: Vec::new(),
            });
        };
        let (n, _) = self.graph.capacity();
        let mut parent_id = vec![None; n];
        let mut closed = vec![false; n];
        let mut open = BinaryHeap::new();

        let dest_pos = self.graph.node_weight(star_b.into()).unwrap().pos;

        open.push(PathfindingNode {
            star: star_a,
            parent: star_a,
            edge_to_parent: u32::MAX,
            origin_dist: 0,
            heuristic_val: (self
                .graph
                .node_weight(star_a.into())
                .unwrap()
                .pos
                .distance(dest_pos)
                * GalaxyConfig::GALACTIC_INTEGER_SCALE as f32) as i32,
        });
        closed[star_a as usize] = true;

        while let Some(top) = open.pop() {
            parent_id[top.star as usize] = Some((top.parent, top.edge_to_parent));

            if top.star == star_b {
                break;
            }

            for n in self.graph.neighbors(top.star.into()) {
                if top.star == star_a && n.index() as u32 == star_b {
                    continue;
                }
                if !closed[n.index()] {
                    closed[n.index()] = true;
                    open.push(PathfindingNode::new(n.index() as u32, &top, dest_pos, self));
                }
            }
        }

        let mut path = Path::new();
        let mut curr = star_b;

        loop {
            path.nodes.push(curr);
            if curr == star_a {
                path.nodes.reverse();
                path.edges.reverse();
                assert!(
                    path.nodes.len() == path.edges.len() + 1,
                    "mismaTched number of path nodes ({}) and edges ({})",
                    path.nodes.len(),
                    path.edges.len()
                );
                return Some(path);
            }
            if let Some(c) = parent_id[curr as usize] {
                path.edges.push(c.1);
                curr = c.0;
            } else {
                return None;
            }
        }
    }

    fn find_path_multi_source(&self, sources: &[u32], star_b: u32) -> Option<Path> {
        let sources_set: std::collections::HashSet<u32> =
            std::collections::HashSet::from_iter(sources.iter().cloned());
        if sources_set.contains(&star_b) {
            return Some(Path {
                nodes: vec![star_b],
                edges: Vec::new(),
            });
        };
        let (n, _) = self.graph.capacity();
        let mut parent_id = vec![None; n];
        let mut closed = vec![false; n];
        let mut open = BinaryHeap::new();

        let dest_pos = self.graph.node_weight(star_b.into()).unwrap().pos;

        for source_star in sources {
            open.push(PathfindingNode {
                star: *source_star,
                parent: *source_star,
                edge_to_parent: u32::MAX,
                origin_dist: 0,
                heuristic_val: (self
                    .graph
                    .node_weight((*source_star).into())
                    .unwrap()
                    .pos
                    .distance(dest_pos)
                    * GalaxyConfig::GALACTIC_INTEGER_SCALE as f32)
                    as i32,
            });
            closed[*source_star as usize] = true;
        }

        while let Some(top) = open.pop() {
            parent_id[top.star as usize] = Some((top.parent, top.edge_to_parent));

            if top.star == star_b {
                break;
            }

            for n in self.graph.neighbors(top.star.into()) {
                if !closed[n.index()] {
                    closed[n.index()] = true;
                    open.push(PathfindingNode::new(n.index() as u32, &top, dest_pos, self));
                }
            }
        }

        let mut path = Path::new();
        let mut curr = star_b;

        loop {
            path.nodes.push(curr);
            if sources_set.contains(&curr) {
                path.nodes.reverse();
                path.edges.reverse();
                assert!(
                    path.nodes.len() == path.edges.len() + 1,
                    "mismaTched number of path nodes ({}) and edges ({})",
                    path.nodes.len(),
                    path.edges.len()
                );
                return Some(path);
            }
            if let Some(c) = parent_id[curr as usize] {
                path.edges.push(c.1);
                curr = c.0;
            } else {
                return None;
            }
        }
    }

    fn find_path(&self, star_a: u32, star_b: u32) -> Option<Path> {
        if star_a == star_b {
            return Some(Path {
                nodes: vec![star_a],
                edges: Vec::new(),
            });
        };
        let (n, _) = self.graph.capacity();
        let mut parent_id = vec![None; n];
        let mut closed = vec![false; n];
        let mut open = BinaryHeap::new();

        let dest_pos = self.graph.node_weight(star_b.into()).unwrap().pos;

        open.push(PathfindingNode {
            star: star_a,
            parent: star_a,
            edge_to_parent: u32::MAX,
            origin_dist: 0,
            heuristic_val: (self
                .graph
                .node_weight(star_a.into())
                .unwrap()
                .pos
                .distance(dest_pos)
                * GalaxyConfig::GALACTIC_INTEGER_SCALE as f32) as i32,
        });
        closed[star_a as usize] = true;

        while let Some(top) = open.pop() {
            parent_id[top.star as usize] = Some((top.parent, top.edge_to_parent));

            if top.star == star_b {
                break;
            }

            for n in self.graph.neighbors(top.star.into()) {
                if !closed[n.index()] {
                    closed[n.index()] = true;
                    open.push(PathfindingNode::new(n.index() as u32, &top, dest_pos, self));
                }
            }
        }

        let mut path = Path::new();
        let mut curr = star_b;

        loop {
            path.nodes.push(curr);
            if curr == star_a {
                path.nodes.reverse();
                path.edges.reverse();
                assert!(
                    path.nodes.len() == path.edges.len() + 1,
                    "mismaTched number of path nodes ({}) and edges ({})",
                    path.nodes.len(),
                    path.edges.len()
                );
                return Some(path);
            }
            if let Some(c) = parent_id[curr as usize] {
                path.edges.push(c.1);
                curr = c.0;
            } else {
                return None;
            }
        }
    }
    ///
    /// Returns vec of distances corresponding to hypernet node ids
    fn dijkstra(&self, input_points: &[u32]) -> Vec<Option<i32>> {
        let (n, _) = self.graph.capacity();
        let mut open = BinaryHeap::new();
        let mut result = vec![None; n];

        for p in input_points {
            open.push(DijkstraNode { star: *p, cost: 0 });
            result[*p as usize] = Some(0);
        }

        while let Some(top) = open.pop() {
            for n in self.graph.neighbors(top.star.into()) {
                if result[n.index()].is_none() {
                    let e = self
                        .graph
                        .edge_weight(self.graph.find_edge(top.star.into(), n).unwrap())
                        .unwrap();
                    result[n.index()] = Some(top.cost + e.length);
                    open.push(DijkstraNode {
                        star: n.index() as u32,
                        cost: top.cost + e.length,
                    });
                }
            }
        }

        result
    }
}
