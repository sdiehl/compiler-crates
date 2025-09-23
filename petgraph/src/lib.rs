use std::collections::HashMap;

use petgraph::algo::{dominators, has_path_connecting, toposort};
use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{Bfs, Dfs, Reversed};

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: String,
    pub instructions: Vec<String>,
}

pub type ControlFlowGraph = DiGraph<BasicBlock, String>;

pub fn build_simple_cfg() -> (ControlFlowGraph, HashMap<String, NodeIndex>) {
    let mut cfg = ControlFlowGraph::new();
    let mut blocks = HashMap::new();

    let entry = cfg.add_node(BasicBlock {
        id: "entry".to_string(),
        instructions: vec!["x = 10".to_string()],
    });
    blocks.insert("entry".to_string(), entry);

    let cond = cfg.add_node(BasicBlock {
        id: "cond".to_string(),
        instructions: vec!["if x > 5".to_string()],
    });
    blocks.insert("cond".to_string(), cond);

    let then_block = cfg.add_node(BasicBlock {
        id: "then".to_string(),
        instructions: vec!["x = x * 2".to_string()],
    });
    blocks.insert("then".to_string(), then_block);

    let else_block = cfg.add_node(BasicBlock {
        id: "else".to_string(),
        instructions: vec!["x = x + 1".to_string()],
    });
    blocks.insert("else".to_string(), else_block);

    let merge = cfg.add_node(BasicBlock {
        id: "merge".to_string(),
        instructions: vec!["print(x)".to_string()],
    });
    blocks.insert("merge".to_string(), merge);

    cfg.add_edge(entry, cond, "fallthrough".to_string());
    cfg.add_edge(cond, then_block, "true".to_string());
    cfg.add_edge(cond, else_block, "false".to_string());
    cfg.add_edge(then_block, merge, "fallthrough".to_string());
    cfg.add_edge(else_block, merge, "fallthrough".to_string());

    (cfg, blocks)
}

pub fn build_loop_cfg() -> (ControlFlowGraph, HashMap<String, NodeIndex>) {
    let mut cfg = ControlFlowGraph::new();
    let mut blocks = HashMap::new();

    let entry = cfg.add_node(BasicBlock {
        id: "entry".to_string(),
        instructions: vec!["i = 0".to_string()],
    });
    blocks.insert("entry".to_string(), entry);

    let loop_header = cfg.add_node(BasicBlock {
        id: "loop_header".to_string(),
        instructions: vec!["if i < 10".to_string()],
    });
    blocks.insert("loop_header".to_string(), loop_header);

    let loop_body = cfg.add_node(BasicBlock {
        id: "loop_body".to_string(),
        instructions: vec!["sum += i".to_string(), "i += 1".to_string()],
    });
    blocks.insert("loop_body".to_string(), loop_body);

    let exit = cfg.add_node(BasicBlock {
        id: "exit".to_string(),
        instructions: vec!["return sum".to_string()],
    });
    blocks.insert("exit".to_string(), exit);

    cfg.add_edge(entry, loop_header, "fallthrough".to_string());
    cfg.add_edge(loop_header, loop_body, "true".to_string());
    cfg.add_edge(loop_header, exit, "false".to_string());
    cfg.add_edge(loop_body, loop_header, "backedge".to_string());

    (cfg, blocks)
}

pub fn perform_dfs(graph: &ControlFlowGraph, start: NodeIndex) -> Vec<NodeIndex> {
    let mut dfs = Dfs::new(&graph, start);
    let mut visited = Vec::new();

    while let Some(node) = dfs.next(&graph) {
        visited.push(node);
    }

    visited
}

pub fn perform_bfs(graph: &ControlFlowGraph, start: NodeIndex) -> Vec<NodeIndex> {
    let mut bfs = Bfs::new(&graph, start);
    let mut visited = Vec::new();

    while let Some(node) = bfs.next(&graph) {
        visited.push(node);
    }

    visited
}

pub fn find_dominators(
    graph: &ControlFlowGraph,
    entry: NodeIndex,
) -> HashMap<NodeIndex, NodeIndex> {
    let dom_tree = dominators::simple_fast(&graph, entry);
    let mut dom_map = HashMap::new();

    for node in graph.node_indices() {
        if let Some(idom) = dom_tree.immediate_dominator(node) {
            if idom != node {
                dom_map.insert(node, idom);
            }
        }
    }

    dom_map
}

pub fn detect_unreachable_code(graph: &ControlFlowGraph, entry: NodeIndex) -> Vec<NodeIndex> {
    let mut unreachable = Vec::new();

    for node in graph.node_indices() {
        if !has_path_connecting(&graph, entry, node, None) {
            unreachable.push(node);
        }
    }

    unreachable
}

pub fn topological_ordering(graph: &ControlFlowGraph) -> Option<Vec<NodeIndex>> {
    toposort(&graph, None).ok()
}

pub fn print_cfg_dot(graph: &ControlFlowGraph) -> String {
    format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]))
}

#[derive(Debug, Clone)]
pub struct CallGraphNode {
    pub name: String,
    pub is_recursive: bool,
}

pub type CallGraph = DiGraph<CallGraphNode, ()>;

pub fn build_call_graph() -> (CallGraph, HashMap<String, NodeIndex>) {
    let mut cg = CallGraph::new();
    let mut funcs = HashMap::new();

    let main = cg.add_node(CallGraphNode {
        name: "main".to_string(),
        is_recursive: false,
    });
    funcs.insert("main".to_string(), main);

    let parse = cg.add_node(CallGraphNode {
        name: "parse".to_string(),
        is_recursive: false,
    });
    funcs.insert("parse".to_string(), parse);

    let analyze = cg.add_node(CallGraphNode {
        name: "analyze".to_string(),
        is_recursive: false,
    });
    funcs.insert("analyze".to_string(), analyze);

    let codegen = cg.add_node(CallGraphNode {
        name: "codegen".to_string(),
        is_recursive: false,
    });
    funcs.insert("codegen".to_string(), codegen);

    let optimize = cg.add_node(CallGraphNode {
        name: "optimize".to_string(),
        is_recursive: true,
    });
    funcs.insert("optimize".to_string(), optimize);

    cg.add_edge(main, parse, ());
    cg.add_edge(main, analyze, ());
    cg.add_edge(main, codegen, ());
    cg.add_edge(analyze, optimize, ());
    cg.add_edge(optimize, optimize, ());

    (cg, funcs)
}

pub fn find_recursive_functions(graph: &CallGraph) -> Vec<NodeIndex> {
    let mut recursive = Vec::new();

    for node in graph.node_indices() {
        // Check for self-loops (direct recursion)
        if graph.find_edge(node, node).is_some() {
            recursive.push(node);
            continue;
        }

        // Check for indirect recursion (cycles that include this node)
        let mut dfs = Dfs::new(graph, node);
        while let Some(visited) = dfs.next(graph) {
            if visited != node && graph.find_edge(visited, node).is_some() {
                recursive.push(node);
                break;
            }
        }
    }

    recursive
}

pub fn reverse_postorder(graph: &ControlFlowGraph, entry: NodeIndex) -> Vec<NodeIndex> {
    let reversed = Reversed(&graph);
    let mut dfs = Dfs::new(&reversed, entry);
    let mut stack = Vec::new();

    while let Some(node) = dfs.next(&reversed) {
        stack.push(node);
    }

    stack.reverse();
    stack
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfg_construction() {
        let (cfg, blocks) = build_simple_cfg();
        assert_eq!(cfg.node_count(), 5);
        assert_eq!(cfg.edge_count(), 5);
        assert!(blocks.contains_key("entry"));
        assert!(blocks.contains_key("merge"));
    }

    #[test]
    fn test_dfs_traversal() {
        let (cfg, blocks) = build_simple_cfg();
        let entry = blocks["entry"];
        let visited = perform_dfs(&cfg, entry);
        assert_eq!(visited.len(), 5);
    }

    #[test]
    fn test_dominators() {
        let (cfg, blocks) = build_simple_cfg();
        let entry = blocks["entry"];
        let doms = find_dominators(&cfg, entry);

        let cond = blocks["cond"];
        let merge = blocks["merge"];
        assert_eq!(doms[&cond], entry);
        assert_eq!(doms[&merge], cond);
    }

    #[test]
    fn test_loop_detection() {
        let (cfg, _) = build_loop_cfg();
        let topo_order = topological_ordering(&cfg);
        assert!(topo_order.is_none());
    }

    #[test]
    fn test_recursive_functions() {
        let (cg, _funcs) = build_call_graph();
        let recursive = find_recursive_functions(&cg);
        assert_eq!(recursive.len(), 1);
        assert_eq!(cg[recursive[0]].name, "optimize");
    }
}
