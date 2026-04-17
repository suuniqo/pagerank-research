mod parser;
mod partition;

use parser::{Parser, ParseError};

#[derive(Debug, Clone)]
pub struct Graph {
    n_nodes: usize,
    n_edges: usize,
    adj_list: Vec<Vec<usize>>
}

impl Graph {
    pub fn from_mtx(path: &str) -> Result<Self, ParseError> {
        let (metadata, edges) = Parser::parse_mtx(path)?;
        let mut adj_list = vec![vec![]; metadata.nrows];

        for (src, dst) in edges {
            adj_list[src].push(dst);
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

    pub fn successors_unchecked(&self, v: usize) -> &[usize] {
        &self.adj_list[v]
    }

    pub fn degree_unchecked(&self, v: usize) -> usize {
        self.adj_list[v].len()
    }

    pub fn make_undirected(&self) -> Self {
        let mut n_edges = self.n_edges;
        let mut adj_list = self.adj_list.clone();

        for (u, adj) in self.adj_list.iter().enumerate() {
            for &v in adj {
                if !adj_list[v].contains(&u) {
                    adj_list[v].push(u);
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
}
