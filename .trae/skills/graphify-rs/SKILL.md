---
name: "graphify-rs"
description: "Turn any codebase into a navigable knowledge graph using the high-performance graphify-rs engine. Invoke when user wants to extract AST/Markdown relations, analyze architecture, or asks to run /graphify commands."
---

# /graphify-rs

Turn any folder of files (especially novel drafts, worldbuilding notes, and character profiles) into a navigable knowledge graph with blazingly fast Markdown parsing, community detection, and AI semantic extraction.

## Usage

```bash
/graphify-rs                               # full pipeline on current directory
/graphify-rs <path>                        # full pipeline on specific path
/graphify-rs query "<question>"            # answer a question using the story graph context
/graphify-rs path "<CharA>" "<CharB>"      # find connections and conflicts between two characters/events
/graphify-rs explain "<EntityName>"        # explain a specific character/location and its relationships
```

## What graphify-rs is for

`graphify-rs` is built around the concept of instantly understanding complex narratives and worldbuilding. Drop it into your novel's repository, and get a structured knowledge graph that shows you hidden character arcs, unresolved conflicts, and lore connections.

Three things it does that Claude alone cannot:
1. **Persistent story graph** - relationships are extracted natively and stored in `graphify-out-rs/graph.json`. You can query character motivations without reading thousands of words into context.
2. **Deterministic Speed** - It uses `pulldown-cmark` to parse your Markdown notes and `[[Wiki Links]]` in milliseconds.
3. **Cross-document surprise** - Built-in Leiden community detection finds connections between characters in different subplots that you would never think to ask about directly.

Use it for:
- A complex novel with many subplots (understand the web of relationships)
- Discovering "God Nodes" (the most central characters or pivotal events)
- Finding "Surprising Connections" (how seemingly unrelated factions or events interact)

## What You Must Do When Invoked

Follow these steps in order based on the user's command.

### Step 1 - Ensure graphify-rs is installed

Check if `graphify-rs` is in the system PATH. If not, try to download the pre-compiled binary. If that fails or `cargo` is preferred, install it globally using Cargo.

```bash
if ! command -v graphify-rs &> /dev/null; then
    echo "graphify-rs not found. Installing..."
    
    # Try downloading pre-compiled binary first (requires curl and jq)
    if command -v curl &> /dev/null && command -v jq &> /dev/null; then
        OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
        ARCH="$(uname -m)"
        
        if [ "$OS" = "darwin" ]; then
            if [ "$ARCH" = "arm64" ]; then ASSET_NAME="graphify-rs-macos-aarch64"
            else ASSET_NAME="graphify-rs-macos-x86_64"; fi
        elif [ "$OS" = "linux" ]; then
            ASSET_NAME="graphify-rs-linux-x86_64"
        fi
        
        if [ -n "$ASSET_NAME" ]; then
            echo "Downloading $ASSET_NAME..."
            DOWNLOAD_URL=$(curl -s https://api.github.com/repos/xfwgithub/graphify-rs/releases/latest | jq -r ".assets[] | select(.name == \"$ASSET_NAME\") | .browser_download_url")
            if [ -n "$DOWNLOAD_URL" ] && [ "$DOWNLOAD_URL" != "null" ]; then
                curl -L -o /usr/local/bin/graphify-rs "$DOWNLOAD_URL" || curl -L -o ~/.cargo/bin/graphify-rs "$DOWNLOAD_URL" || curl -L -o /tmp/graphify-rs "$DOWNLOAD_URL"
                
                # Make executable and link if downloaded to tmp
                if [ -f "/tmp/graphify-rs" ]; then
                    chmod +x /tmp/graphify-rs
                    mkdir -p ~/.local/bin
                    mv /tmp/graphify-rs ~/.local/bin/graphify-rs
                    export PATH="$HOME/.local/bin:$PATH"
                else
                    chmod +x /usr/local/bin/graphify-rs 2>/dev/null || chmod +x ~/.cargo/bin/graphify-rs 2>/dev/null
                fi
                echo "Successfully downloaded pre-compiled binary."
            fi
        fi
    fi
    
    # Fallback to cargo install if the binary download failed or wasn't attempted
    if ! command -v graphify-rs &> /dev/null; then
        if command -v cargo &> /dev/null; then
            echo "Falling back to Cargo installation..."
            if [ -f "Cargo.toml" ] && grep -q 'name = "graphify-rs"' Cargo.toml; then
                cargo install --path .
            else
                cargo install --git https://github.com/xfwgithub/graphify-rs.git
            fi
        else
            echo "Error: Neither pre-compiled binary could be downloaded nor Cargo is available."
            exit 1
        fi
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

Read the key novel draft files or notes in the target directory and generate a valid JSON file named `.graphify_semantic.json` containing:
```json
{
  "nodes": [
    {"id": "unique_id", "label": "Human Readable Name", "kind": "character|event|location|faction|item", "properties": {}, "pagerank": 0.0}
  ],
  "edges": [
    {"source": "node_id", "target": "node_id", "kind": "allies_with|enemies_with|loves|betrays|participates_in|belongs_to|occurs_at", "weight": 0.8}
  ]
}
```
*Note: Focus on extracting narrative arcs, character relationships, and conflicts that aren't explicitly linked via Markdown `[[links]]`.*

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

### Step 6 - Present the STORY GRAPH REPORT

You must generate a clear, Markdown-formatted report for the user based on the JSON data you just read. Your report MUST include:

1. **Story Corpus Summary**: How many entities (characters/events) and relationship edges were found.
2. **Pivotal Nodes (God Nodes)**: Identify the top 3-5 characters or events with the highest `pagerank` or highest degree. Explain why they are central to the narrative web (e.g., "The central antagonist connecting 3 different subplots").
3. **Factions & Story Arcs (Communities)**: Look at how nodes cluster together. Group them into logical "Factions", "Locations", or "Subplots" based on their connectivity, and assign them a 2-5 word plain-language name (e.g. "The Royal Court Intrigue", "The Rebellion Camp").
4. **Narrative Insights & Unresolved Conflicts**: Provide 1-2 bullet points highlighting surprising connections or potential plot holes based on the graph (e.g., "Character A and C are deeply connected through Event B, but haven't interacted directly").

Always maintain an authoritative, analytical tone. You are an expert editor and story architect analyzing the narrative topology.