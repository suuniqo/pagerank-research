use std::process;

pub mod graph;

use graph::{Graph, painter::Painter, partition::LouvainBuilder};

use crate::graph::parser::Parser;

fn _test_stanford() {
    let start = std::time::Instant::now();

    let graph = match Graph::from_mtx("data/web-stanford/web-Stanford.mtx") {
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

fn main() {
    let graph_tsv = match Parser::parse_tsv(
        "data/wikispeedia/articles.tsv",
        "data/wikispeedia/categories.tsv",
        "data/wikispeedia/links.tsv",
    ) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1);
        },
    };

    dbg!(graph_tsv.edges.len());
    dbg!(graph_tsv.nodes.len());
    dbg!(&graph_tsv.ids.iter().take(10).collect::<Vec<(&String, &usize)>>());
    dbg!(&graph_tsv.nodes[..10]);
    dbg!(&graph_tsv.edges[..10]);
    dbg!(&graph_tsv.categories[..10]);

}
