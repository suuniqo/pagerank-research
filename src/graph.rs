mod parser;

use parser::{Parser, ParseError};

#[derive(Debug, Clone, Copy)]
pub struct GraphMetadata {
    pub nrows: usize,
    pub ncols: usize,
    pub nnz: usize,
}

impl GraphMetadata {
    pub fn new(nrows: usize, ncols: usize, nnz: usize) -> Self {
        Self {
            nrows,
            ncols,
            nnz,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Graph {
    metadata: GraphMetadata,
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
            metadata,
            adj_list
        })
    }

    pub fn successors(&self, v: usize) -> &[usize] {
        &self.adj_list[v]
    }
}
