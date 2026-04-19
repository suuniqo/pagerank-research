mod parser;
pub mod partition;

use parser::{Parser, ParseError};

#[derive(Debug, Clone)]
pub struct Graph {
    n_nodes: usize,
    n_edges: usize,
    adj_list: Vec<Vec<(usize, usize)>>
}

impl Graph {
    pub fn from_mtx(path: &str) -> Result<Self, ParseError> {
        let (metadata, edges) = Parser::parse_mtx(path)?;
        let mut adj_list = vec![vec![]; metadata.nrows];

        for (src, dst) in edges {
            adj_list[src].push((dst, 1));
        }

        Ok(Self {
            n_nodes: metadata.nrows,
            n_edges: metadata.nnz,
            adj_list
        })
    }

    pub fn n_nodes(&self) -> usize {
        self.n_nodes
    }

    pub fn n_edges(&self) -> usize {
        self.n_edges
    }

    pub fn successors_unchecked(&self, v: usize) -> &[(usize, usize)] {
        &self.adj_list[v]
    }

    pub fn degree_unchecked(&self, v: usize) -> usize {
        self.adj_list[v].len()
    }

    pub fn strength_unchecked(&self, v: usize) -> usize {
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
