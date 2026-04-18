use crate::models::{Edge, Node};
use anyhow::Result;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use regex::Regex;
use std::path::Path;

pub struct ExtractResult {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

pub fn extract_markdown(path: &Path, content: &str) -> Result<ExtractResult> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    
    let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
    let str_path = path.to_string_lossy().to_string();
    
    // File node
    let file_id = sanitize_id(&stem);
    nodes.push(Node {
        id: file_id.clone(),
        label: stem.clone(),
        kind: "document".to_string(),
        properties: serde_json::json!({
            "source_file": str_path,
        }),
        pagerank: 0.0,
    });

    // Simple YAML frontmatter parsing
    if content.starts_with("---\n") {
        if let Some(end) = content[4..].find("\n---\n") {
            let frontmatter = &content[4..4+end];
            if let Ok(docs) = yaml_rust::YamlLoader::load_from_str(frontmatter) {
                if let Some(doc) = docs.first() {
                    // Extract title if exists
                    if let Some(title) = doc["title"].as_str() {
                        nodes[0].label = title.to_string();
                    }
                    
                    // Link source nodes from memory Q&A
                    if let Some(source_nodes) = doc["source_nodes"].as_vec() {
                        for sn in source_nodes {
                            if let Some(sn_str) = sn.as_str() {
                                edges.push(Edge {
                                    source: file_id.clone(),
                                    target: sn_str.to_string(),
                                    kind: "references".to_string(),
                                    weight: 1.0,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Parse markdown headers
    let parser = Parser::new(content);
    let mut current_header = String::new();
    let mut in_header = false;

    // Detect wiki-links like [[Some Page]]
    let wikilink_re = Regex::new(r"\[\[(.*?)\]\]").unwrap();

    for event in parser {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                in_header = true;
                current_header.clear();
            }
            Event::Text(text) => {
                if in_header {
                    current_header.push_str(&text);
                }
                
                // Extract wiki-links
                for cap in wikilink_re.captures_iter(&text) {
                    if let Some(link) = cap.get(1) {
                        let link_str = link.as_str().to_string();
                        let target_id = sanitize_id(&link_str);
                        
                        edges.push(Edge {
                            source: file_id.clone(),
                            target: target_id,
                            kind: "mentions".to_string(),
                            weight: 0.8,
                        });
                    }
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                in_header = false;
                let header_str = current_header.trim().to_string();
                if !header_str.is_empty() {
                    let header_id = sanitize_id(&format!("{}_{}", stem, header_str));
                    
                    nodes.push(Node {
                        id: header_id.clone(),
                        label: header_str,
                        kind: "section".to_string(),
                        properties: serde_json::json!({
                            "source_file": str_path,
                        }),
                        pagerank: 0.0,
                    });
                    
                    edges.push(Edge {
                        source: file_id.clone(),
                        target: header_id,
                        kind: "contains".to_string(),
                        weight: 1.0,
                    });
                }
            }
            _ => {}
        }
    }

    Ok(ExtractResult { nodes, edges })
}

fn sanitize_id(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
    let cleaned = re.replace_all(input, "_");
    cleaned.trim_matches('_').to_lowercase()
}
