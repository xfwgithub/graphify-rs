---
name: "graphify-rs"
description: "Turn any codebase into a navigable knowledge graph using the high-performance graphify-rs engine. Invoke when user wants to extract AST/Markdown relations, analyze architecture, or asks to run /graphify commands."
---

# /graphify-rs

Turn any folder of files into a navigable knowledge graph with blazingly fast AST/Markdown parsing, community detection, and AI semantic extraction.

## Usage

```bash
/graphify-rs                               # full pipeline on current directory
/graphify-rs <path>                        # full pipeline on specific path
/graphify-rs query "<question>"            # answer a question using the graph context
/graphify-rs path "<NodeA>" "<NodeB>"      # find connections between two concepts
/graphify-rs explain "<NodeName>"          # explain a specific node and its relationships
```

## What You Must Do When Invoked

Follow these steps in order based on the user's command.

### Step 1 - Ensure graphify-rs is installed

Check if `graphify-rs` is in the system PATH. If not, install it globally using Cargo so it can be run from anywhere.

```bash
if ! command -v graphify-rs &> /dev/null; then
    echo "Installing graphify-rs globally..."
    if [ -f "Cargo.toml" ] && grep -q 'name = "graphify-rs"' Cargo.toml; then
        cargo install --path .
    else
        cargo install --git https://github.com/xfwgithub/graphify-rs.git
    fi
fi
```

### Step 2 - Parse Intent & Handle Queries

If the user invoked a read-only command (`query`, `path`, or `explain`), DO NOT run the full extraction again if `graphify-out-rs/graph.json` already exists. 
Instead, read the JSON file and answer the user directly based on the nodes and edges:
- **query**: Traverse the graph to answer the architectural question.
- **path**: Find the shortest or most relevant path between the two nodes in the JSON.
- **explain**: List the incoming/outgoing edges of the specific node and summarize its role.
*Stop here if the user invoked one of these commands.*

### Step 3 - Extract Semantic Relationships (AI-assisted)

If the user invoked the full pipeline (no subcommand), you must first generate the implicit semantic edges.

Read the key files in the target directory and generate a valid JSON file named `.graphify_semantic.json` containing:
```json
{
  "nodes": [
    {"id": "unique_id", "label": "Human Readable Name", "kind": "concept|function|class", "properties": {}, "pagerank": 0.0}
  ],
  "edges": [
    {"source": "node_id", "target": "node_id", "kind": "conceptually_related_to|calls|implements", "weight": 0.8}
  ]
}
```
*Note: Focus only on finding deep, cross-file architectural or semantic connections that simple ASTs miss.*

### Step 4 - Run Extraction & Import Semantics

Run the extraction on the target directory, importing the semantic JSON you just created.

```bash
TARGET_PATH="${1:-.}"
graphify-rs --target "$TARGET_PATH" --import-semantic .graphify_semantic.json --out ./graphify-out-rs
```

### Step 5 - Read and Analyze the Graph

Read the generated `graph.json`. DO NOT print the raw JSON to the user. Instead, use tools like `jq` or `cat` combined with your context window to analyze the graph structure.

```bash
cat ./graphify-out-rs/graph.json | jq '{nodes: (.nodes | length), edges: (.edges | length)}'
```

### Step 6 - Present the GRAPH REPORT

You must generate a clear, Markdown-formatted report for the user based on the JSON data you just read. Your report MUST include:

1. **Corpus Summary**: How many nodes and edges were found.
2. **God Nodes**: Identify the top 3-5 nodes with the highest `pagerank` (if available) or the highest degree (most incoming/outgoing edges). Explain what role they likely play in the system (e.g., "Core configuration", "Main event loop").
3. **Community Structure**: Look at how nodes cluster together. Group them into logical "Modules" based on their connectivity or naming conventions, and assign them a 2-5 word plain-language name (e.g. "AST Parsing Module", "Graph Analysis Core").
4. **Architectural Insights**: Provide 1-2 bullet points explaining how the codebase is structured based on the graph.

Always maintain an authoritative, analytical tone. You are a senior software architect presenting the codebase topology.