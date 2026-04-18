use crate::models::{Edge, Node};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Deserialize, Debug)]
struct SemNode {
    id: String,
    label: String,
    kind: String,
}

#[derive(Deserialize, Debug)]
struct SemEdge {
    source: String,
    target: String,
    kind: String,
    weight: f64,
}

#[derive(Deserialize, Debug)]
struct SemanticResult {
    #[serde(default)]
    nodes: Vec<SemNode>,
    #[serde(default)]
    edges: Vec<SemEdge>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    response_format: Option<ResponseFormat>,
}

#[derive(Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    format_type: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatMessage {
    content: String,
}

pub struct ExtractResult {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

pub async fn extract_semantic(file_path: &str, content: &str) -> Result<ExtractResult> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow!("OPENAI_API_KEY environment variable is not set"))?;
    let api_base = env::var("OPENAI_API_BASE").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()); // Default model

    let client = reqwest::Client::new();

    let prompt = format!(
        r#"You are a knowledge graph extraction agent. Read the file below and extract a knowledge graph.
Output ONLY valid JSON matching this schema:
{{
  "nodes": [
    {{"id": "unique_id", "label": "Human Readable Name", "kind": "concept|function|class"}}
  ],
  "edges": [
    {{"source": "node_id", "target": "node_id", "kind": "calls|implements|references|conceptually_related_to", "weight": 1.0}}
  ]
}}
Do not output markdown code blocks, just raw JSON.

File path: {}
Content:
{}
"#,
        file_path, content
    );

    let req_body = ChatRequest {
        model,
        messages: vec![
            Message {
                role: "system".to_string(),
                content: "You are a graph extraction AI. Return only JSON.".to_string(),
            },
            Message {
                role: "user".to_string(),
                content: prompt,
            },
        ],
        response_format: Some(ResponseFormat {
            format_type: "json_object".to_string(),
        }),
    };

    let res = client
        .post(format!("{}/chat/completions", api_base))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req_body)
        .send()
        .await?;

    if !res.status().is_success() {
        let err_text = res.text().await?;
        return Err(anyhow!("LLM API Error: {}", err_text));
    }

    let chat_res: ChatResponse = res.json().await?;
    let json_str = chat_res.choices.first()
        .ok_or_else(|| anyhow!("No choices in response"))?
        .message.content.clone();

    // Clean markdown code blocks if the model accidentally included them
    let json_str = json_str.trim();
    let json_str = if json_str.starts_with("```json") {
        let end = json_str.len().saturating_sub(3);
        &json_str[7..end]
    } else if json_str.starts_with("```") {
        let end = json_str.len().saturating_sub(3);
        &json_str[3..end]
    } else {
        json_str
    };

    let sem_res: SemanticResult = serde_json::from_str(json_str)?;

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for sn in sem_res.nodes {
        nodes.push(Node {
            id: sn.id,
            label: sn.label,
            kind: sn.kind,
            properties: serde_json::json!({"source_file": file_path}),
            pagerank: 0.0,
        });
    }

    for se in sem_res.edges {
        edges.push(Edge {
            source: se.source,
            target: se.target,
            kind: se.kind,
            weight: se.weight,
        });
    }

    Ok(ExtractResult { nodes, edges })
}
