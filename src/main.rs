use std::process;

pub mod graph;

use graph::{Graph, painter::Painter, partition::LouvainBuilder};

const DATA_PATH: &str = "data/web-Stanford.mtx";

fn main() {
    let start = std::time::Instant::now();

    let graph = match Graph::from_mtx(DATA_PATH) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
    };

    let elapsed = start.elapsed();
    println!("process matrix: {} ms", elapsed.as_millis());

    let undirected = graph.make_undirected();

    let start = std::time::Instant::now();

    let partition = LouvainBuilder::new(&undirected)
        .fast(true)
        .resolution(1.0)
        .gain_threshold(0.0001)
        .run();

    let elapsed = start.elapsed();
    println!("louvain method: {} ms", elapsed.as_millis());

    let mut communities: Vec<usize> = partition.communities().into_iter().map(|c| c.len()).collect();
    communities.sort_by(|c1, c2| c2.cmp(c1));

    let n_comm = communities.len();

    println!();
    println!("REPORT:");
    println!("- communities: \t{}", partition.len());
    println!("- modularity: \t{}", partition.modularity());
    println!("- largest: \t{:?}", &communities[..5.min(n_comm)]);
    println!("- smallest: \t{:?}", &communities[n_comm.saturating_sub(5)..]);

    Painter::draw_aggregate(&partition, "out/aggregate.dot");
}
