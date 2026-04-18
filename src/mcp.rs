use anyhow::Result;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub async fn run_server() -> Result<()> {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin).lines();

    while let Ok(Some(line)) = reader.next_line().await {
        if line.trim().is_empty() {
            continue;
        }

        let request: Result<Value, _> = serde_json::from_str(&line);
        
        let response = match request {
            Ok(req) => handle_request(req).await,
            Err(_) => {
                Some(json!({
                    "jsonrpc": "2.0",
                    "error": { "code": -32700, "message": "Parse error" },
                    "id": Value::Null
                }))
            }
        };

        if let Some(res) = response {
            let res_str = format!("{}\n", serde_json::to_string(&res).unwrap());
            stdout.write_all(res_str.as_bytes()).await?;
            stdout.flush().await?;
        }
    }
    
    Ok(())
}

async fn handle_request(req: Value) -> Option<Value> {
    let id = req.get("id").cloned().unwrap_or(Value::Null);
    let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

    // Minimal JSON-RPC JSON format mapping
    let result = match method {
        "initialize" => {
            json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "graphify-rs",
                    "version": "0.1.0"
                },
                "capabilities": {
                    "tools": {}
                }
            })
        },
        "tools/list" => {
            json!({
                "tools": [
                    {
                        "name": "graphify_extract",
                        "description": "Extract knowledge graph from directory",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "path": { "type": "string" }
                            },
                            "required": ["path"]
                        }
                    },
                    {
                        "name": "graphify_search",
                        "description": "Search the knowledge graph",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": { "type": "string" }
                            },
                            "required": ["query"]
                        }
                    }
                ]
            })
        },
        _ => {
            return Some(json!({
                "jsonrpc": "2.0",
                "error": { "code": -32601, "message": "Method not found" },
                "id": id
            }));
        }
    };

    Some(json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id
    }))
}
