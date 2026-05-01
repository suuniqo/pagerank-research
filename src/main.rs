use std::{collections::HashMap, process};

pub mod graph;
pub mod matrix;
pub mod parser;

use parser::GraphTSV;
use graph::{Graph, painter::Painter, partition::{LouvainBuilder, PartitionSet}};

use crate::matrix::Matrix;

fn _test_lvn_stanford() {
    let start = std::time::Instant::now();

    let (graph, _) = match Graph::from_mtx("data/web-stanford/links.mtx") {
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


fn _test_lvn_wikispeedia() {
    fn community_frequencies(info: &GraphTSV, partitions: &PartitionSet) -> Vec<(HashMap<String, usize>, usize)> {
        let mut frequencies = vec![(HashMap::new(), 0); partitions.len()];
        
        for node in 0..info.nodes.len() {
            let comm = partitions.community(node);

            let (temp, count) = &mut frequencies[comm];

            for category in &info.categories[node] {
                // for word in category {
                if let Some(word) = category.first() {
                    temp.entry(word.to_string()).and_modify(|x| {*x += 1}).or_insert(1);
                    *count += 1;
                }
            }

        }

        frequencies
    }

    let start = std::time::Instant::now();

    let (graph, tsv_info) = match Graph::from_tsv(
        "data/wikispeedia/articles.tsv",
        "data/wikispeedia/categories.tsv",
        "data/wikispeedia/links.tsv"
    ) {
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
        .resolution(2.5)
        .gain_threshold(0.0001)
        .run();

    let elapsed = start.elapsed();
    println!("louvain method: {} ms", elapsed.as_millis());

    let communities: Vec<usize> = partition.communities().into_iter().map(|c| c.len()).collect();
    let mut community_size_ord = communities.clone();
    community_size_ord.sort_by(|c1, c2| c2.cmp(c1));

    let n_comm = communities.len();

    println!();
    println!("REPORT:");
    println!("- communities: \t{}", partition.len());
    println!("- modularity: \t{}", partition.modularity());
    println!("- largest: \t{:?}", &community_size_ord[..5.min(n_comm)]);
    println!("- smallest: \t{:?}", &community_size_ord[n_comm.saturating_sub(5)..]);

    println!("\nCOMMUNITIES:");
    let frequencies = community_frequencies(&tsv_info, &partition);

    for (comm, (comm_f, total)) in frequencies.into_iter().enumerate() {
        let size = communities[comm];

        let total = total as f64;
        let mut sorted = comm_f.into_iter()
            .map(|(word, count)| (word, (count as f64) / total))
            .collect::<Vec<_>>();
        sorted.sort_by(|(_, x), (_, y)| y.partial_cmp(x).unwrap());

        
        let formatted: Vec<String> = sorted.iter()
            .map(|(name, x)| format!("({name}, {:.2})", x))
            .collect();

        println!("size: {size} \ttags: {:?}", &formatted[..4.min(sorted.len())]);
    }

    Painter::draw_aggregate(&partition, "out/wikispeedia/aggregate.dot");
}

fn _test_pgr_stanford() {
    let start = std::time::Instant::now();

    let (mat, _) = match Matrix::from_mtx("data/web-stanford/links.mtx") {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
    };

    let elapsed = start.elapsed();
    println!("process matrix: {} ms", elapsed.as_millis());

    let start = std::time::Instant::now();

    let (rank, precision) = mat.pagerank(0.85, 50);

    let elapsed = start.elapsed();
    println!("pagerank: {} ms", elapsed.as_millis());

    let sum = rank.sum();

    let mut rank = rank.iter().map(|v| v / sum).collect::<Vec<f64>>();

    rank.sort_by(|a, b| b.partial_cmp(&a).unwrap());

    println!();
    println!("REPORT:");
    println!("- precision: \t{}", precision);
    println!("- ranking: \t{:?}", &rank[..10]);
}

fn _test_pgr_wikispeedia() {
    let start = std::time::Instant::now();

    let (mat, tsv_info) = match Matrix::from_tsv(
        "data/wikispeedia/articles.tsv",
        "data/wikispeedia/categories.tsv",
        "data/wikispeedia/links.tsv"
    ) {
        Ok(g) => g,
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(1);
        }
    };

    let elapsed = start.elapsed();
    println!("process matrix: {} ms", elapsed.as_millis());

    let start = std::time::Instant::now();

    let (rank, precision) = mat.pagerank(0.85, 100);

    let elapsed = start.elapsed();
    println!("pagerank: {} ms", elapsed.as_millis());

    let sum = rank.sum();

    let mut rank = rank
        .iter()
        .enumerate()
        .map(|(i, r)| (i, *r))
        .collect::<Vec<(usize, f64)>>();

    rank.sort_by(|(_, a), (_, b)| b.partial_cmp(&a).unwrap());

    let named_ranks: Vec<(String, f64)> = rank
        .iter()
        .map(|(i, rank)| (tsv_info.nodes[*i].clone(), *rank / sum))
        .collect();

    println!();
    println!("REPORT:");
    println!("- precision: \t{}", precision);
    println!("- ranking sum: \t{}", sum);
    println!("- ranking: \t{{");
    for nr in named_ranks.iter().take(10) {
        println!("\t{:?}", nr);
    }
    println!("}}");
}

fn main() {
    _test_pgr_wikispeedia();
}
