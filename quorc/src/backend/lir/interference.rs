use std::collections::{HashMap, HashSet};

use crate::{backend::lir::regalloc::LiveRange, midend::mir::block::VReg};

#[derive(Debug, Clone, Default)]
pub struct InterferenceGraph {
    pub nodes: HashSet<VReg>,
    pub edges: HashMap<VReg, HashSet<VReg>>,
}

impl InterferenceGraph {
    pub fn add_node(&mut self, vreg: VReg) {
        self.nodes.insert(vreg);
        self.edges.entry(vreg).or_default();
    }

    pub fn add_edge(&mut self, v1: VReg, v2: VReg) {
        if v1 == v2 {
            return;
        }

        self.add_node(v1);
        self.add_node(v2);

        self.edges.get_mut(&v1).unwrap().insert(v2);
        self.edges.get_mut(&v2).unwrap().insert(v1);
    }

    pub fn neighbors(&self, vreg: &VReg) -> &HashSet<VReg> {
        self.edges.get(vreg).unwrap()
    }

    pub fn degree(&self, vreg: &VReg) -> usize {
        self.neighbors(vreg).len()
    }

    pub fn remove_node(&mut self, vreg: &VReg) {
        if let Some(neighbors) = self.edges.get(vreg) {
            let neighbors_copy = neighbors.clone();
            for neighbor in neighbors_copy {
                if let Some(neighbor_edges) = self.edges.get_mut(&neighbor) {
                    neighbor_edges.remove(vreg);
                }
            }
        }

        self.nodes.remove(vreg);
        self.edges.remove(vreg);
    }
}

pub fn build_interference_graph(live_ranges: &HashMap<VReg, LiveRange>) -> InterferenceGraph {
    let mut graph = InterferenceGraph::default();

    for vreg in live_ranges.keys() {
        graph.add_node(*vreg);
    }

    let ranges: Vec<_> = live_ranges.values().collect();
    for i in 0..ranges.len() {
        for j in i + 1..ranges.len() {
            if ranges_interfere(ranges[i], ranges[j]) {
                graph.add_edge(ranges[i].vreg, ranges[j].vreg);
            }
        }
    }

    graph
}

fn ranges_interfere(r1: &LiveRange, r2: &LiveRange) -> bool {
    r1.start <= r2.end && r2.start <= r1.end
}
