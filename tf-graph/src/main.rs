use std::path::PathBuf;

use clap::Parser;
use color_eyre::Result;
use teralib::TFGraph;

mod graphviz;

#[derive(Parser, Debug)]
struct Arguments {
    #[clap(default_value = "./")]
    root_dir: PathBuf,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Arguments::parse();

    let mut types = ignore::types::TypesBuilder::new();
    // types.add_defaults();
    types.add("hcl", "*.lock.hcl")?;
    types.add("tfstate", "*.tfstate")?;
    let types = types.select("tfstate").select("hcl").build()?;

    let walker = ignore::WalkBuilder::new(args.root_dir)
        .hidden(false) // This means we are not ignoring hidden files........
        .types(types)
        .build();

    let terraform_roots: Vec<_> = walker
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                // We only care about lock files
                return None;
            }

            path.parent().map(|p| p.to_path_buf())
        })
        .collect();

    let graph = TFGraph::default();
    let graph = terraform_roots
        .iter()
        .fold(graph, |graph, root| graph.with_root(root));

    let graph = graph.build()?;
    println!("/*\n{:#?}\n*/", graph);
    let output = graphviz::output_graphviz(&graph);
    println!("{}", output);

    Ok(())
}
