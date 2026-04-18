use crate::models::{Edge, Node};
use anyhow::Result;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

pub struct GraphManager {
    graph: DiGraph<Node, Edge>,
    node_map: HashMap<String, NodeIndex>,
}

impl GraphManager {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(&node.id) {
            return idx;
        }
        let id = node.id.clone();
        let idx = self.graph.add_node(node);
        self.node_map.insert(id, idx);
        idx
    }

    pub fn add_edge(&mut self, source_id: &str, target_id: &str, edge: Edge) {
        if let (Some(&src), Some(&tgt)) = (self.node_map.get(source_id), self.node_map.get(target_id)) {
            self.graph.add_edge(src, tgt, edge);
        }
    }

    pub fn compute_pagerank(&mut self, iterations: usize, damping_factor: f64) {
        let num_nodes = self.graph.node_count() as f64;
        if num_nodes == 0.0 {
            return;
        }

        // Initialize PageRank values
        let initial_pr = 1.0 / num_nodes;
        let mut pr_values: Vec<f64> = vec![initial_pr; self.graph.node_count()];

        for _ in 0..iterations {
            let mut new_pr = vec![(1.0 - damping_factor) / num_nodes; self.graph.node_count()];

            for i in 0..self.graph.node_count() {
                let idx = NodeIndex::new(i);
                let out_degree = self.graph.edges_directed(idx, petgraph::Direction::Outgoing).count() as f64;
                
                if out_degree > 0.0 {
                    let share = pr_values[i] / out_degree;
                    for edge in self.graph.edges_directed(idx, petgraph::Direction::Outgoing) {
                        new_pr[petgraph::visit::EdgeRef::target(&edge).index()] += damping_factor * share;
                    }
                } else {
                    // Handle dangling nodes
                    for j in 0..self.graph.node_count() {
                        new_pr[j] += damping_factor * (pr_values[i] / num_nodes);
                    }
                }
            }
            pr_values = new_pr;
        }

        // Write back
        for (i, node) in self.graph.node_weights_mut().enumerate() {
            node.pagerank = pr_values[i];
        }
    }

    pub fn export_json(&self, file_path: &str) -> Result<()> {
        use std::fs::File;
        use std::io::Write;

        let nodes: Vec<&Node> = self.graph.node_weights().collect();
        let edges: Vec<&Edge> = self.graph.edge_weights().collect();

        let god_nodes = crate::analyze::find_god_nodes(&self.graph, 10);
        
        // Run clustering
        let communities = crate::cluster::cluster(&self.graph);
        
        let surprising_connections = crate::analyze::find_surprising_connections(&self.graph, 5, &communities);

        let export_data = serde_json::json!({
            "nodes": nodes,
            "edges": edges,
            "analysis": {
                "god_nodes": god_nodes,
                "surprising_connections": surprising_connections,
            }
        });

        let mut file = File::create(file_path)?;
        file.write_all(serde_json::to_string_pretty(&export_data)?.as_bytes())?;
        Ok(())
    }
}
