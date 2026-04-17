mod graph;

use graph::Graph;

use crate::graph::partition::PartitionSet;

const DATA_PATH: &str = "data/web-Stanford.mtx";

fn main() {
    let graph = match Graph::from_mtx(DATA_PATH) {
        Ok(g) => g,
        Err(err) => panic!("{err}")
    };

    let undirected = graph.make_undirected();

    let partition = PartitionSet::singleton(&undirected);

    dbg!(partition.modularity());
}
