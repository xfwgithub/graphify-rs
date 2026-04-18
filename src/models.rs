use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub properties: serde_json::Value,
    pub pagerank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub kind: String,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GodNode {
    pub id: String,
    pub label: String,
    pub edges: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurprisingConnection {
    pub source: String,
    pub target: String,
    pub source_files: Vec<String>,
    pub relation: String,
    pub confidence: String,
    pub why: String,
}
