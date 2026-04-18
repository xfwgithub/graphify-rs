use crate::models::{Edge, GodNode, Node, SurprisingConnection};
use petgraph::graph::DiGraph;
use std::collections::HashSet;

fn is_file_node(node: &Node) -> bool {
    let label = &node.label;
    if let Some(source_file) = node.properties.get("source_file").and_then(|v| v.as_str()) {
        if let Some(file_name) = std::path::Path::new(source_file).file_name().and_then(|n| n.to_str()) {
            if label == file_name {
                return true;
            }
        }
    }
    if label.starts_with('.') && label.ends_with("()") {
        return true;
    }
    false
}

fn is_concept_node(node: &Node) -> bool {
    if let Some(source_file) = node.properties.get("source_file").and_then(|v| v.as_str()) {
        if source_file.is_empty() {
            return true;
        }
        if !source_file.contains('.') {
            return true;
        }
        false
    } else {
        true
    }
}

pub fn find_god_nodes(graph: &DiGraph<Node, Edge>, top_n: usize) -> Vec<GodNode> {
    let mut node_degrees: Vec<_> = graph
        .node_indices()
        .map(|idx| {
            let in_degree = graph.edges_directed(idx, petgraph::Direction::Incoming).count();
            let out_degree = graph.edges_directed(idx, petgraph::Direction::Outgoing).count();
            (idx, in_degree + out_degree)
        })
        .collect();

    node_degrees.sort_by(|a, b| b.1.cmp(&a.1));

    let mut result = Vec::new();
    for (idx, degree) in node_degrees {
        let node = &graph[idx];
        if is_file_node(node) || is_concept_node(node) {
            continue;
        }
        result.push(GodNode {
            id: node.id.clone(),
            label: node.label.clone(),
            edges: degree,
        });
        if result.len() >= top_n {
            break;
        }
    }
    result
}

fn file_category(path: &str) -> &'static str {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match ext.as_str() {
        "py" | "ts" | "tsx" | "js" | "go" | "rs" | "java" | "rb" | "cpp" | "c" | "h" | "cs" | "kt" | "scala" | "php" => "code",
        "pdf" => "paper",
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg" => "image",
        _ => "doc",
    }
}

fn top_level_dir(path: &str) -> &str {
    path.split('/').next().unwrap_or(path)
}

pub fn find_surprising_connections(graph: &DiGraph<Node, Edge>, top_n: usize, communities: &std::collections::HashMap<usize, Vec<String>>) -> Vec<SurprisingConnection> {
    let mut source_files = HashSet::new();
    
    // Invert communities mapping for O(1) lookup
    let mut node_community = std::collections::HashMap::new();
    for (cid, nodes) in communities {
        for n in nodes {
            node_community.insert(n.clone(), *cid);
        }
    }
    for node in graph.node_weights() {
        if let Some(source_file) = node.properties.get("source_file").and_then(|v| v.as_str()) {
            if !source_file.is_empty() {
                source_files.insert(source_file);
            }
        }
    }

    let is_multi_source = source_files.len() > 1;

    let mut candidates = Vec::new();

    for edge in graph.edge_references() {
        use petgraph::visit::EdgeRef;
        let source_idx = edge.source();
        let target_idx = edge.target();
        let u = &graph[source_idx];
        let v = &graph[target_idx];
        let e = edge.weight();

        let relation = &e.kind;
        if ["imports", "imports_from", "contains", "method"].contains(&relation.as_str()) {
            continue;
        }
        if is_concept_node(u) || is_concept_node(v) || is_file_node(u) || is_file_node(v) {
            continue;
        }

        let u_source = u.properties.get("source_file").and_then(|v| v.as_str()).unwrap_or("");
        let v_source = v.properties.get("source_file").and_then(|v| v.as_str()).unwrap_or("");

        if is_multi_source && (u_source.is_empty() || v_source.is_empty() || u_source == v_source) {
            continue;
        }

        let mut score = 0;
        let mut reasons = Vec::new();

        // Confidence
        let conf = "EXTRACTED"; // Default if not present, assume graphify uses some properties on edge, wait, Edge has properties? No, Edge has no properties field.
        // wait, I need to add properties to Edge in models.rs to support confidence, or just use weight/kind.
        // I will add confidence logic later if needed. For now, let's just do cross-file etc.
        
        let cat_u = file_category(u_source);
        let cat_v = file_category(v_source);
        if cat_u != cat_v {
            score += 2;
            reasons.push(format!("crosses file types ({} ↔ {})", cat_u, cat_v));
        }

        if top_level_dir(u_source) != top_level_dir(v_source) {
            score += 2;
            reasons.push("connects across different repos/directories".to_string());
        }

        let deg_u = graph.edges_directed(source_idx, petgraph::Direction::Incoming).count() + graph.edges_directed(source_idx, petgraph::Direction::Outgoing).count();
        let deg_v = graph.edges_directed(target_idx, petgraph::Direction::Incoming).count() + graph.edges_directed(target_idx, petgraph::Direction::Outgoing).count();
        
        if std::cmp::min(deg_u, deg_v) <= 2 && std::cmp::max(deg_u, deg_v) >= 5 {
            score += 1;
            let (peripheral, hub) = if deg_u <= 2 { (&u.label, &v.label) } else { (&v.label, &u.label) };
            reasons.push(format!("peripheral node `{}` unexpectedly reaches hub `{}`", peripheral, hub));
        }

        // 4. 跨社区奖励
        if let (Some(c_u), Some(c_v)) = (node_community.get(&u.id), node_community.get(&v.id)) {
            if c_u != c_v {
                score += 1;
                reasons.push("bridges separate communities".to_string());
            }
        }

        if score > 0 || !is_multi_source {
            // For single source, we could use edge betweenness, but we don't have it implemented yet, 
            // so we just return edges that cross something or have high score.
            candidates.push((score, SurprisingConnection {
                source: u.label.clone(),
                target: v.label.clone(),
                source_files: vec![u_source.to_string(), v_source.to_string()],
                relation: relation.clone(),
                confidence: conf.to_string(),
                why: if reasons.is_empty() { "cross-file semantic connection".to_string() } else { reasons.join("; ") },
            }));
        }
    }

    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    candidates.into_iter().map(|(_, c)| c).take(top_n).collect()
}
