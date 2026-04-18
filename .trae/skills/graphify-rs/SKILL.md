---
name: "graphify-rs"
description: "Turn any codebase into a navigable knowledge graph using the high-performance graphify-rs engine. Invoke when user wants to extract AST/Markdown relations, analyze architecture, or asks to run /graphify commands."
---

# /graphify-rs

This skill uses `graphify-rs` (the ultra-fast Rust port of graphify) to scan the current codebase and generate a knowledge graph (`graph.json`) containing code AST and Markdown relations.

## Usage

```bash
/graphify-rs                          # Full pipeline on current directory
/graphify-rs <path>                   # Full pipeline on specific path
```

## What You Must Do When Invoked

Follow these steps in order. Do not skip steps.

### Step 1 - Ensure graphify-rs is compiled

Check if the release binary exists. If not, compile it:

```bash
if [ ! -f "target/release/graphify-rs" ]; then
    echo "Compiling graphify-rs..."
    cargo build --release
fi
```

### Step 2 - Run Extraction

Run the extraction on the target directory (defaulting to `.` if no path was provided).

```bash
TARGET_PATH="${1:-.}"
./target/release/graphify-rs --target "$TARGET_PATH" --out ./graphify-out-rs
```

### Step 3 - Analyze Results

Read the generated `graph.json` and provide a high-level summary of the codebase architecture to the user based on the extracted nodes and edges.

```bash
cat ./graphify-out-rs/graph.json | jq '{nodes: .nodes | length, edges: .edges | length}'
```

Based on the JSON data, answer any specific architectural questions the user asked.