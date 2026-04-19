mod graph;

use graph::Graph;

use crate::graph::partition::PartitionSet;

const DATA_PATH: &str = "data/web-Stanford.mtx";

fn main() {
    let start = std::time::Instant::now();
    let graph = match Graph::from_mtx(DATA_PATH) {
        Ok(g) => g,
        Err(err) => panic!("{err}")
    };
    let elapsed = start.elapsed();
    println!("process matrix: {}ms", elapsed.as_millis());

    let undirected = graph.make_undirected();

    let start = std::time::Instant::now();
    let communities = PartitionSet::from_louvain(&undirected, 1.0, true, None, None);
    let elapsed = start.elapsed();
    println!("louvain method: {}ms", elapsed.as_millis());

    println!();
    println!("REPORT:");
    println!("- communities: \t{}", communities.len());
    println!("- modularity: \t{}", communities.modularity());
}
