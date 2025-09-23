use petgraph_example::{
    build_call_graph, build_loop_cfg, build_simple_cfg, find_dominators, find_recursive_functions,
    perform_bfs, perform_dfs, print_cfg_dot, reverse_postorder, topological_ordering,
};

fn main() {
    println!("=== Control Flow Graph Example ===");
    let (cfg, blocks) = build_simple_cfg();

    println!(
        "CFG has {} nodes and {} edges",
        cfg.node_count(),
        cfg.edge_count()
    );

    let entry = blocks["entry"];
    println!();
    println!("DFS traversal from entry:");
    let dfs_order = perform_dfs(&cfg, entry);
    for node in &dfs_order {
        println!("  - {}", cfg[*node].id);
    }

    println!();
    println!("BFS traversal from entry:");
    let bfs_order = perform_bfs(&cfg, entry);
    for node in &bfs_order {
        println!("  - {}", cfg[*node].id);
    }

    println!();
    println!("Dominator relationships:");
    let doms = find_dominators(&cfg, entry);
    for (node, idom) in &doms {
        println!("  {} is dominated by {}", cfg[*node].id, cfg[*idom].id);
    }

    println!();
    println!("=== Loop CFG Example ===");
    let (loop_cfg, loop_blocks) = build_loop_cfg();

    println!(
        "Loop CFG has {} nodes and {} edges",
        loop_cfg.node_count(),
        loop_cfg.edge_count()
    );

    println!();
    println!(
        "Topological sort possible: {}",
        topological_ordering(&loop_cfg).is_some()
    );

    let loop_entry = loop_blocks["entry"];
    println!();
    println!("Reverse postorder traversal:");
    let rpo = reverse_postorder(&loop_cfg, loop_entry);
    for node in &rpo {
        println!("  - {}", loop_cfg[*node].id);
    }

    println!();
    println!("=== Call Graph Example ===");
    let (call_graph, _funcs) = build_call_graph();

    println!("Call graph has {} functions", call_graph.node_count());

    println!();
    println!("Recursive functions:");
    let recursive = find_recursive_functions(&call_graph);
    for node in &recursive {
        println!("  - {}", call_graph[*node].name);
    }

    println!();
    println!("=== DOT Output ===");
    println!("CFG in DOT format:");
    println!("{}", print_cfg_dot(&cfg));
}
