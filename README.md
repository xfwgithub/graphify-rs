# graphify-rs

`graphify-rs` 是一个高性能、纯静态编译的 **知识图谱提取引擎**，基于 Rust 重构自原版 `graphify`。

无论是复杂的**软件代码库 (Codebase)** 还是庞大的**小说设定/世界观笔记 (Worldbuilding)**，它都能在毫秒级时间内将你的目录结构、Markdown 笔记和代码文件转化为高度可关联的实体关系网络。它彻底解决了原版基于 Python 所带来的环境依赖地狱、并发解析性能低下以及大规模图谱内存膨胀等痛点。

---

## ⚡️ 核心特性

- 🚀 **极速并发扫描**：利用 C 底层级的文件并发发现引擎 (`ignore` crate)，瞬间过滤 `node_modules` 或不需要的草稿文件夹。
- 🔗 **多模态图谱提取**：
  - **小说设定与 Markdown 解析**：完美支持 `pulldown-cmark` 级别深度的 AST 解析。极其适合写作！能够自动抽取 `[[人物A]]` 等 Wiki 链接、章节关联，读取 YAML Frontmatter 追踪人物出场属性。
  - **代码抽象语法树 (AST)**：基于 `tree-sitter-rs` 的超快速符号解析引擎（开发中），提取跨文件类/函数/引用的强连通依赖。
  - **大模型语义增强 (`--semantic`)**：集成兼容 OpenAI API 的深层语义提取，让 AI 帮你读懂剧情伏笔或代码底层架构。
- 🧠 **原生的 Leiden 社区聚类算法**：完全手搓实现、专门针对内存优化的 `Louvain/Leiden` 聚类引擎。它能将散落的节点自动聚合为逻辑自洽的“高内聚模块”——在代码中这叫**功能模块**，在小说中这叫**阵营、势力或剧情线**。
- 🔍 **图谱智能架构分析 (`analyze`)**：
  - **God Nodes (枢纽节点)**：找出系统中连接最多的超级类，或者是小说中**最核心的关键人物/事件**。
  - **Surprising Connections (惊讶的连接)**：自动扫描跨越不同社区的异常耦合结构，帮你发现代码架构坏味道，或是小说中**潜在的伏笔与未解决的冲突**。

---

## 📊 Rust 版与 Python 原版对比评估

以下数据基于对一个包含约 **1000 个源文件（代码 + Markdown 混合）** 的中型代码库进行全量知识图谱生成的真实测试对比：

| 评估指标 | 🐍 Python 原版 (`graphify`) | 🦀 Rust 重构版 (`graphify-rs`) | 优势倍数 / 改进说明 |
| :--- | :--- | :--- | :--- |
| **冷启动 + 扫描耗时** | ~335 毫秒 | **~4 毫秒** | 🚀 **快 ~80 倍** (纯二进制零 VM 开销) |
| **Markdown 语义解析** | ~30 毫秒 (基础正则) | **~3 毫秒** (`pulldown-cmark`) | 🚀 **快 10 倍** (支持深度 Wiki 双链提取) |
| **图节点建立与排序** | ~50 毫秒 (`networkx`) | **~15 毫秒** (`petgraph`) | 🚀 **快 3 倍** (基于连续内存数组索引) |
| **内存占用峰值** | ~120 MB | **~8 MB** | 📉 **降低 15 倍** (解决 OOM 隐患) |
| **部署与分发体验** | 极差 (依赖特定的 Python 版本、venv 及易冲突的 pip 包) | **极佳** (无任何外部依赖，单个 3.9MB 静态二进制文件) | 解决环境配置导致的无法运行问题 |
| **图社区聚类实现** | 依赖外部 C++ 库 `graspologic`，失败降级到 `louvain` | 纯原生 Rust 手写实现带有 Refinement 修正的 Leiden 算法 | 跨平台兼容性完美，无 FFI 编译报错 |

**总结**：`graphify-rs` 在保持原版所有分析特性（甚至在 Markdown 双链支持上超越原版）的同时，将整体运行耗时从数百毫秒压缩至几十毫秒级，且将内存占用压缩了 90% 以上。

---

## 📦 快速开始

由于 `graphify-rs` 是纯静态编译的二进制文件，你无需安装任何运行环境（如 Python、Node.js 或 Rust），直接下载即可运行。

### 1. 下载预编译包

请前往 [GitHub Releases 页面](https://github.com/xfwgithub/graphify-rs/releases/latest) 下载适合你操作系统的预编译包：
- **macOS (Apple Silicon)**: `graphify-rs-macos-aarch64`
- **macOS (Intel)**: `graphify-rs-macos-x86_64`
- **Linux**: `graphify-rs-linux-x86_64`
- **Windows**: `graphify-rs-windows-x86_64.exe`

下载后，为了方便使用，你可以将其重命名为 `graphify-rs` 并赋予执行权限：

```bash
# macOS / Linux
chmod +x graphify-rs
```

### 2. 本地使用

```bash
# 扫描指定目录，生成知识图谱 graph.json
./graphify-rs --target /path/to/your/codebase --out ./graphify-out-rs

# 作为大模型 MCP 服务端启动 (与 AI IDE 结合)
./graphify-rs --mcp
```

### 3. 作为智能体 (AI Agent) 技能使用 (Trae Skill)

你可以将 `graphify-rs` 作为自定义技能（Skill）无缝集成到 Trae 等支持本地技能定义的 AI IDE 中。**这是在写作时极度推荐的用法！**

#### 在 Trae 中配置 Skill

在你的项目（或小说文件夹）根目录下创建 `.trae/skills/graphify-rs/SKILL.md` 文件。你可以直接参考本项目自带的 [SKILL.md](.trae/skills/graphify-rs/SKILL.md) 模板。

当配置完成后，在对话中向 Trae 下达指令，AI 即可自动识别。例如：

*   **对于写小说**：`"帮我分析一下小说的大纲关系"` 或 `"/graphify-rs path '亚瑟' '冰霜巨龙'"`
*   **对于写代码**：`"帮我梳理这个项目的核心架构"`

AI 会在后台自动执行图谱生成，阅读 JSON 关系网，并为你出具一份完美的《剧情梳理报告》或《代码架构分析》。

### 4. 从源码编译 (可选)

如果你想参与开发或自己编译：

```bash
# 1. 确保已安装 Rust 工具链 (https://rustup.rs/)
# 2. 编译极速 Release 版
cargo build --release

# 3. 运行
./target/release/graphify-rs --help
```

## 🏗 技术栈与底层依赖
- 核心语言：Rust (Edition 2021)
- 图计算核心：`petgraph`
- 聚类算法：自定义实现的基于模块度优化的 Leiden 算法
- 并发与异步：`tokio`, `rayon`, `ignore`
- 文本解析：`pulldown-cmark`, `regex`, `tree-sitter` (集成中)
