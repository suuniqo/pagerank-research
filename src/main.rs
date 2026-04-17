mod graph;

use graph::Graph;

const DATA_PATH: &str = "data/web-Stanford.mtx";

fn main() {
    let graph = match Graph::from_mtx(DATA_PATH) {
        Ok(g) => g,
        Err(err) => panic!("{err}")
    };

    dbg!(graph.successors(871));
    dbg!(graph.successors(1));

    let undirected = graph.make_undirected();

    dbg!(undirected.successors(871));
    dbg!(undirected.successors(1));
}
