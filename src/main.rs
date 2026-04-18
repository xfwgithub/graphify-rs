use anyhow::Result;
use clap::Parser;

mod models;
mod detect;
mod analyze;
mod graph;
mod extract_md;
mod cluster;
mod mcp;
mod semantic;

#[derive(Parser, Debug)]
#[command(author, version, about = "Graphify in Rust - Fast codebase knowledge graph", long_about = None)]
struct Args {
    /// 目标分析目录 (Target directory to analyze)
    #[arg(short, long, default_value = ".")]
    target: String,
    
    /// 输出目录 (Output directory)
    #[arg(short, long, default_value = "graphify-out")]
    out: String,

    /// 运行 MCP 服务端 (Run as MCP server)
    #[arg(long)]
    mcp: bool,

    /// 开启 LLM 语义提取 (Enable semantic extraction via LLM)
    #[arg(long)]
    semantic: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    if args.mcp {
        // Run MCP server loop
        mcp::run_server().await?;
        return Ok(());
    }

    println!("🚀 [Graphify-rs] 开始分析目录: {}", args.target);
    
    let files = detect::scan_directory(&args.target)?;
    println!("📦 找到 {} 个受支持的代码文件", files.len());
    
    if args.semantic && std::env::var("OPENAI_API_KEY").is_err() {
        println!("⚠️ 警告: 开启了 --semantic 但未设置 OPENAI_API_KEY 环境变量。语义提取将被跳过。");
    }

    let mut graph_manager = graph::GraphManager::new();
    
    // Add file nodes and parse Markdown
    for file in &files {
        let name = file.to_string_lossy().to_string();
        let content = std::fs::read_to_string(file).unwrap_or_default();
        
        if name.ends_with(".md") {
            if let Ok(res) = extract_md::extract_markdown(file, &content) {
                for node in res.nodes {
                    graph_manager.add_node(node);
                }
                for edge in res.edges {
                    graph_manager.add_edge(&edge.source, &edge.target, edge.clone());
                }
                continue;
            }
        }
        
        // Semantic Extraction (LLM-based) if enabled
        if args.semantic && std::env::var("OPENAI_API_KEY").is_ok() {
            // 只对文本内容较小的文件或者非二进制文件进行语义提取
            if !content.is_empty() && content.len() < 100_000 {
                println!("🧠 正在使用大模型语义提取: {}", name);
                match semantic::extract_semantic(&name, &content).await {
                    Ok(res) => {
                        for node in res.nodes {
                            graph_manager.add_node(node);
                        }
                        for edge in res.edges {
                            graph_manager.add_edge(&edge.source.clone(), &edge.target.clone(), edge);
                        }
                        continue;
                    }
                    Err(e) => {
                        println!("❌ 语义提取失败 [{}]: {}", name, e);
                    }
                }
            }
        }

        // Fallback for non-markdown, failed parsing, or non-semantic mode
        graph_manager.add_node(models::Node {
            id: name.clone(),
            label: name.clone(),
            kind: "file".to_string(),
            properties: serde_json::json!({}),
            pagerank: 0.0,
        });
    }

    graph_manager.compute_pagerank(10, 0.85);
    
    let out_dir = std::path::Path::new(&args.out);
    if !out_dir.exists() {
        std::fs::create_dir_all(out_dir)?;
    }
    
    let out_file = out_dir.join("graph.json");
    graph_manager.export_json(out_file.to_str().unwrap())?;
    
    println!("✨ 知识图谱已生成: {}", out_file.display());
    
    Ok(())
}
