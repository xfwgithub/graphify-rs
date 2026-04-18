use crate::models::{Edge, Node};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use rand::seq::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;

/// 简单的 Louvain 算法实现 (Rust 原生平替)
/// 用于图的社区发现，返回 {community_id: [node_ids]}
pub fn cluster(graph: &DiGraph<Node, Edge>) -> HashMap<usize, Vec<String>> {
    let mut communities: HashMap<usize, Vec<String>> = HashMap::new();
    let num_nodes = graph.node_count();

    if num_nodes == 0 {
        return communities;
    }

    // 初始化：每个节点属于一个独立的社区
    let mut node_to_com: HashMap<NodeIndex, usize> = HashMap::new();
    for i in 0..num_nodes {
        let idx = NodeIndex::new(i);
        node_to_com.insert(idx, i);
    }

    // 计算图的总边权重 (2m)
    let m2: f64 = graph.edge_weights().map(|e| e.weight).sum::<f64>();
    if m2 == 0.0 {
        // 如果没有边，直接返回所有节点作为独立社区
        for i in 0..num_nodes {
            let idx = NodeIndex::new(i);
            let id = graph[idx].id.clone();
            communities.entry(i).or_default().push(id);
        }
        return communities;
    }

    // 预计算每个节点的度数 (入度+出度权重之和)
    let mut degrees: HashMap<NodeIndex, f64> = HashMap::new();
    for i in 0..num_nodes {
        let idx = NodeIndex::new(i);
        let mut deg = 0.0;
        for edge in graph.edges_directed(idx, petgraph::Direction::Outgoing) {
            deg += edge.weight().weight;
        }
        for edge in graph.edges_directed(idx, petgraph::Direction::Incoming) {
            deg += edge.weight().weight;
        }
        degrees.insert(idx, deg);
    }

    // 简单迭代优化模块度 (Louvain Pass 1)
    let mut improved = true;
    let mut max_iters = 10; // 防止死循环

    while improved && max_iters > 0 {
        improved = false;
        max_iters -= 1;

        // 随机遍历节点
        let mut nodes: Vec<NodeIndex> = (0..num_nodes).map(NodeIndex::new).collect();
        let mut rng = StdRng::seed_from_u64(42);
        nodes.shuffle(&mut rng);

        for &u in &nodes {
            let current_com = node_to_com[&u];
            let k_i = degrees[&u];

            // 统计相邻社区的权重总和
            let mut com_weights: HashMap<usize, f64> = HashMap::new();
            for edge in graph.edges_directed(u, petgraph::Direction::Outgoing) {
                let v = edge.target();
                if u != v {
                    let c_v = node_to_com[&v];
                    *com_weights.entry(c_v).or_default() += edge.weight().weight;
                }
            }
            for edge in graph.edges_directed(u, petgraph::Direction::Incoming) {
                let v = edge.source();
                if u != v {
                    let c_v = node_to_com[&v];
                    *com_weights.entry(c_v).or_default() += edge.weight().weight;
                }
            }

            // 计算当前社区的总度数 (近似)
            // 寻找能带来最大模块度增益的社区
            let mut best_com = current_com;
            let mut max_delta = 0.0;

            for (&c, &k_i_in) in &com_weights {
                if c == current_com { continue; }

                let sigma_tot: f64 = node_to_com.iter()
                    .filter(|&(_, &com)| com == c)
                    .map(|(&n, _)| degrees[&n])
                    .sum();

                let delta = k_i_in - (sigma_tot * k_i) / m2;

                if delta > max_delta {
                    max_delta = delta;
                    best_com = c;
                }
            }

            // 移动节点
            if max_delta > 0.0 && best_com != current_com {
                node_to_com.insert(u, best_com);
                improved = true;
            }
        }
    }

    // --- Leiden 算法的核心：Refinement (细化) 阶段 ---
    // 为了防止出现不连通的社区，Leiden 在移动节点后，会尝试在当前社区内部
    // 再次进行局部的、随机的合并，确保合并后的超级节点内部是强连通的。
    // 这里我们实现一个简化的 Refinement 逻辑：将不连通的子图拆分。

    let mut refined_node_to_com = node_to_com.clone();
    let mut next_new_com_id = num_nodes; // 用于分配新的拆分社区ID

    // 1. 按当前分配的社区分组节点
    let mut com_to_nodes: HashMap<usize, Vec<NodeIndex>> = HashMap::new();
    for (&n, &c) in &node_to_com {
        com_to_nodes.entry(c).or_default().push(n);
    }

    // 2. 对每个社区，检查内部连通性
    for (c, com_nodes) in com_to_nodes {
        if com_nodes.len() <= 1 {
            continue; // 单节点社区必然连通
        }

        // 构建当前社区的子图连通分量 (这里简单使用 BFS/DFS)
        let mut visited: HashSet<NodeIndex> = HashSet::new();
        let mut components: Vec<Vec<NodeIndex>> = Vec::new();

        for &start_node in &com_nodes {
            if !visited.contains(&start_node) {
                let mut comp = Vec::new();
                let mut queue = vec![start_node];
                visited.insert(start_node);

                while let Some(curr) = queue.pop() {
                    comp.push(curr);
                    // 遍历所有邻居，只看同属当前社区的
                    for edge in graph.edges_directed(curr, petgraph::Direction::Outgoing) {
                        let nxt = edge.target();
                        if node_to_com[&nxt] == c && !visited.contains(&nxt) {
                            visited.insert(nxt);
                            queue.push(nxt);
                        }
                    }
                    for edge in graph.edges_directed(curr, petgraph::Direction::Incoming) {
                        let nxt = edge.source();
                        if node_to_com[&nxt] == c && !visited.contains(&nxt) {
                            visited.insert(nxt);
                            queue.push(nxt);
                        }
                    }
                }
                components.push(comp);
            }
        }

        // 如果该社区分裂成了多个不连通的分量，说明 Louvain 产生了 Bug
        // Leiden 的要求是必须拆分它们！
        if components.len() > 1 {
            // 第一个分量保留原社区 ID，其余的分配新 ID
            for (i, comp) in components.iter().enumerate().skip(1) {
                let new_cid = next_new_com_id + i;
                for &n in comp {
                    refined_node_to_com.insert(n, new_cid);
                }
            }
            next_new_com_id += components.len();
        }
    }
    
    // 用 Refine 后的结果覆盖
    node_to_com = refined_node_to_com;
    // ------------------------------------------------

    // 整理结果
    for (idx, com) in node_to_com {
        let id = graph[idx].id.clone();
        communities.entry(com).or_default().push(id);
    }

    // 重排社区 ID (按大小降序，和原版一致)
    let mut com_list: Vec<Vec<String>> = communities.into_values().collect();
    com_list.sort_by_key(|c| std::cmp::Reverse(c.len()));

    let mut final_communities = HashMap::new();
    for (i, mut com) in com_list.into_iter().enumerate() {
        com.sort(); // 保证稳定性
        final_communities.insert(i, com);
    }

    final_communities
}
