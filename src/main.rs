mod graph;

use graph::{Graph, painter::Painter};

use crate::graph::partition::LouvainBuilder;

const DATA_PATH: &str = "data/web-Stanford.mtx";

fn main() {
    let start = std::time::Instant::now();

    let graph = match Graph::from_mtx(DATA_PATH) {
        Ok(g) => g,
        Err(err) => panic!("{err}")
    };

    let elapsed = start.elapsed();
    println!("process matrix: {} ms", elapsed.as_millis());

    let undirected = graph.make_undirected();

    let start = std::time::Instant::now();

    let communities = LouvainBuilder::new(&undirected)
        .fast(true)
        .resolution(1.0)
        .gain_threshold(0.0001)
        .run();

    let elapsed = start.elapsed();
    println!("louvain method: {} ms", elapsed.as_millis());

    println!();
    println!("REPORT:");
    println!("- communities: \t{}", communities.len());
    println!("- modularity: \t{}", communities.modularity());

    Painter::draw_graph(&communities.aggregate_graph(), "out/aggregate.dot");
    Painter::draw_partition(&communities, "out/partition.dot");
}
