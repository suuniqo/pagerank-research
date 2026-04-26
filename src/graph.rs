pub mod painter;
pub mod parser;
pub mod partition;

use parser::{Parser, ParseError};

#[derive(Debug, Clone)]
pub struct Graph {
    n_nodes: usize,
    n_edges: usize,
    adj_list: Vec<Vec<(usize, usize)>>
}

impl Graph {
    pub fn new(adj_list: Vec<Vec<(usize, usize)>>) -> Self {
        let n_edges = adj_list
            .iter()
            .map(|adj| adj.len())
            .sum::<usize>() / 2;

        let n_nodes = adj_list.len();

        Self {
            n_nodes,
            n_edges,
            adj_list,
        }
    }

    pub fn from_mtx(path: &str) -> Result<Self, ParseError> {
        let graph = Parser::parse_mtx(path)?;
        let mut adj_list = vec![vec![]; graph.nrows];

        for (src, dst) in graph.edges {
            adj_list[src].push((dst, 1));
        }

        Ok(Self {
            n_nodes: graph.nrows,
            n_edges: graph.nnz,
            adj_list
        })
    }

    pub fn n_nodes(&self) -> usize {
        self.n_nodes
    }

    pub fn n_edges(&self) -> usize {
        self.n_edges
    }

    pub fn neighbours(&self, u: usize) -> &[(usize, usize)] {
        &self.adj_list[u]
    }

    pub fn degree(&self, u: usize) -> usize {
        self.adj_list[u].len()
    }

    pub fn strength(&self, v: usize) -> usize {
        self.weights_of(v).sum()
    }

    pub fn is_neighbour_of(&self, u: usize, v: usize) -> bool {
        self.adj_list[u].iter().any(|(vv, _)| *vv == v)
    }

    pub fn make_undirected(&self) -> Self {
        let mut n_edges = self.n_edges;
        let mut adj_list: Vec<Vec<(usize, usize)>> = self.adj_list.clone();

        for (u, adj) in self.adj_list.iter().enumerate() {
            for &(v, weight) in adj {
                if !self.is_neighbour_of(v, u) {
                    // TODO: Add more weights
                    adj_list[v].push((u, weight));
                    n_edges += 1;
                }
            }
        }

        Self {
            n_nodes: self.n_nodes,
            n_edges,
            adj_list
        }
    }

    pub fn weights(&self) -> impl Iterator<Item = usize> {
        self.adj_list.iter()
            .map(|adjs| adjs.iter().map(|(_, w)| *w))
            .flatten()
    }

    pub fn weights_of(&self, v: usize) -> impl Iterator<Item = usize> {
        self.adj_list[v].iter().map(|(_, w)| *w)
    }
}
