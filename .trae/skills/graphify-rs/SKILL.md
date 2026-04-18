---
name: "graphify-rs"
description: "使用 graphify-rs 提取代码库知识图谱。当用户需要分析代码架构、提取全量 AST/Markdown 关联关系或生成 graph.json 时调用。"
---

# graphify-rs 知识图谱引擎

此技能使用 `graphify-rs` 极速扫描当前代码库，生成包含代码 AST 和 Markdown 关联的知识图谱 (`graph.json`)。

## 何时使用
- 当用户询问“项目代码架构是怎样的”
- 当用户需要“生成代码库知识图谱”
- 当用户需要提取节点关联、寻找 God Nodes 或 Surprising Connections 时

## 使用方法

1. 确保已下载或编译 `graphify-rs` 二进制文件。
2. 运行命令生成图谱：
   ```bash
   ./target/release/graphify-rs --target . --out ./graphify-out-rs
   ```
3. 分析生成的 `./graphify-out-rs/graph.json`，根据节点之间的连接回答用户的代码架构问题。
