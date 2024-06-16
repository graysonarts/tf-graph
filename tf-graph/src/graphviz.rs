use crate::TFGraph;

pub(crate) fn output_graphviz(graph: &TFGraph) -> String {
    let mut output = String::new();
    output.push_str("digraph G {\n");

    for (id, node) in graph.roots.iter() {
        output.push_str(&format!("  {} [label=\"{}\"];\n", id, node.name(),));
    }

    for (id, _node) in graph.roots.iter() {
        for dep in graph.dependency_list.get(id).unwrap() {
            output.push_str(&format!("  {} -> {};\n", id, dep));
        }
    }

    output.push_str("}\n");

    output
}
