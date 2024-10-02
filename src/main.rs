use egraph_serialize::EGraph;

use anyhow::Context;

use std::io::Write;
use std::path::PathBuf;

use extraction_gym::to_egraph_serialized::get_term;

fn main() {
    env_logger::init();

    let mut extractors = extraction_gym::extractors();
    extractors.retain(|_, ed| ed.use_for_bench);

    let mut args = pico_args::Arguments::from_env();

    let extractor_name: String = args
        .opt_value_from_str("--extractor")
        .unwrap()
        .unwrap_or_else(|| "bottom-up".into());
    if extractor_name == "print" {
        for name in extractors.keys() {
            println!("{}", name);
        }
        return;
    }

    let out_filename: PathBuf = args
        .opt_value_from_str("--out")
        .unwrap()
        .unwrap_or_else(|| "out.json".into());

    let pruned_filename: Option<PathBuf> = args.opt_value_from_str("--pruned").unwrap();

    let filename: String = args.free_from_str().unwrap();

    let rest = args.finish();
    if !rest.is_empty() {
        panic!("Unknown arguments: {:?}", rest);
    }

    let mut out_file = std::fs::File::create(out_filename).unwrap();

    let egraph = EGraph::from_json_file(&filename)
        .with_context(|| format!("Failed to parse {filename}"))
        .unwrap();

    let ed = extractors
        .get(extractor_name.as_str())
        .with_context(|| format!("Unknown extractor: {extractor_name}"))
        .unwrap();

    let start_time = std::time::Instant::now();
    let result = ed.extractor.extract(&egraph, &egraph.root_eclasses);
    let us = start_time.elapsed().as_micros();

    result.check(&egraph);

    if let Some(pruned_filename) = pruned_filename {
        let egraph = get_term(&egraph, &result);
        egraph.to_json_file(pruned_filename.clone()).unwrap();
        println!("Wrote pruned egraph to {}", pruned_filename.display());
    }

    let tree = result.tree_cost(&egraph, &egraph.root_eclasses);
    let dag = result.dag_cost(&egraph, &egraph.root_eclasses);

    log::info!("{filename:40}\t{extractor_name:10}\t{tree:5}\t{dag:5}\t{us:5}");
    writeln!(
        out_file,
        r#"{{ 
    "name": "{filename}",
    "extractor": "{extractor_name}", 
    "tree": {tree}, 
    "dag": {dag}, 
    "micros": {us}
}}"#
    )
    .unwrap();
}

#[cfg(test)]
pub mod test;
