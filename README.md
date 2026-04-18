# graphify-rs

`graphify-rs` 是 `graphify` 的高性能 Rust 重构版本。它可以在毫秒级时间内分析庞大的知识库与代码库，生成带有 `PageRank` 权重和 `Leiden` 社区聚类算法支持的高质量知识图谱。

它没有 Python 繁琐的依赖环境，整个项目编译后是一个小巧的单文件可执行程序，支持跨平台运行，同时内置了标准的大模型上下文协议 (MCP) 服务端，可以直接作为插件接入各种 AI IDE。

## 核心特性
- 🚀 **极速并发扫描**：利用 C 底层级的文件并发发现引擎，瞬间过滤 `node_modules`、`.git` 等无关噪音目录。
- 🔗 **多模态图谱提取**：
  - **Markdown & 考据支持**：支持 `pulldown-cmark` 级别深度的 AST 树解析，能够抽取 `[[Wiki链接]]`、章节标题（Header）关联以及读取 YAML Frontmatter 内存（Q&A 飞轮）。
  - **代码抽象语法树 (AST)**：预留了 `tree-sitter` 插件接口用于提取跨文件类/函数/引用的强连通依赖。
- 🧠 **原生的 Leiden 社区聚类算法**：完全手搓实现的 `Louvain/Leiden` 聚类引擎（支持 `Refinement` 细化防止断连），彻底摆脱了复杂的 C++ 编译绑定，将散落的节点聚合为逻辑自洽的高内聚模块。
- 🔍 **图谱智能分析 (`analyze`)**：
  - **God Nodes**：找出系统中连接最多的枢纽概念或超级类。
  - **Surprising Connections (惊讶的连接)**：自动扫描跨越不同社区、跨代码与论文、周边节点直达中心枢纽的非凡/异常耦合结构，给大模型或人类极强的代码架构洞察力。
- 🔌 **原生 MCP Server 支持**：通过 Stdin/Stdout 直接暴露 JSON-RPC，方便无缝挂载。

## 快速开始

```bash
# 1. 编译极速 Release 版
cargo build --release

# 2. 扫描指定目录，生成 graph.json
./target/release/graphify-rs --target /path/to/your/codebase --out ./graphify-out-rs

# 3. 启动为大模型 MCP 服务端
./target/release/graphify-rs --mcp
```

## 技术栈
- 核心语言：Rust
- 图计算核心：`petgraph`
- 并发与异步：`tokio`, `rayon`, `ignore`
- 文本解析：`pulldown-cmark`, `regex`, `tree-sitter`