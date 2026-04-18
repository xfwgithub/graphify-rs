---
name: "graphify-rs"
description: "Turn any codebase into a navigable knowledge graph using the high-performance graphify-rs engine. Invoke when user wants to extract AST/Markdown relations, analyze architecture, or asks to run /graphify commands."
---

# /graphify-rs

Turn any folder of files into a navigable knowledge graph with blazingly fast AST/Markdown parsing, community detection, and zero Python/LLM overhead.

## Usage

```bash
/graphify-rs                               # full pipeline on current directory
/graphify-rs <path>                        # full pipeline on specific path
```

## What graphify-rs is for

`graphify-rs` is built around the concept of instantly understanding complex codebases. Drop it into any repository, and get a structured knowledge graph that shows you what you didn't know was connected.

Three things it does that Claude alone cannot:
1. **Persistent graph** - relationships are extracted natively and stored in `graphify-out-rs/graph.json`. You can query the architecture without reading thousands of files into context.
2. **Deterministic Speed** - Instead of slow LLM-based semantic extraction, it uses `tree-sitter` and `pulldown-cmark` to parse code and Markdown in milliseconds.
3. **Cross-document surprise** - Built-in Leiden community detection finds connections between concepts in different modules that you would never think to ask about directly.

Use it for:
- A codebase you're new to (understand architecture before touching anything)
- Discovering "God Nodes" (the most central/coupled files in a project)
- Finding "Surprising Connections" (how seemingly unrelated communities interact)

## What You Must Do When Invoked

If no path was given, use `.` (current directory). Do not ask the user for a path.

Follow these steps in order. Do not skip steps.

### Step 1 - Ensure graphify-rs is compiled

Check if the release binary exists. If not, compile it automatically. Do not ask the user.

```bash
if [ ! -f "target/release/graphify-rs" ]; then
    echo "Compiling graphify-rs..."
    cargo build --release
fi
```

### Step 2 - Extract Semantic Relationships (AI-assisted)

Since you (the AI) have the ability to read files and understand semantics, you will generate the implicit semantic edges before running the tool.

Read the key files in the directory and generate a valid JSON file named `.graphify_semantic.json` containing:
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

### Step 3 - Run Extraction & Import Semantics

Run the extraction on the target directory, importing the semantic JSON you just created.

```bash
TARGET_PATH="${1:-.}"
./target/release/graphify-rs --target "$TARGET_PATH" --import-semantic .graphify_semantic.json --out ./graphify-out-rs
```

### Step 4 - Read and Analyze the Graph

Read the generated `graph.json`. DO NOT print the raw JSON to the user. Instead, use tools like `jq` or `cat` combined with your context window to analyze the graph structure.

```bash
cat ./graphify-out-rs/graph.json | jq '{nodes: (.nodes | length), edges: (.edges | length)}'
```

### Step 5 - Present the GRAPH REPORT

You must generate a clear, Markdown-formatted report for the user based on the JSON data you just read. Your report MUST include:

1. **Corpus Summary**: How many nodes and edges were found.
2. **God Nodes**: Identify the top 3-5 nodes with the highest `pagerank` (if available) or the highest degree (most incoming/outgoing edges). Explain what role they likely play in the system (e.g., "Core configuration", "Main event loop").
3. **Community Structure**: Look at how nodes cluster together. Group them into logical "Modules" based on their connectivity or naming conventions, and assign them a 2-5 word plain-language name (e.g. "AST Parsing Module", "Graph Analysis Core").
4. **Architectural Insights**: Provide 1-2 bullet points explaining how the codebase is structured based on the graph.

Always maintain an authoritative, analytical tone. You are a senior software architect presenting the codebase topology.