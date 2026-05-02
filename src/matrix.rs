use faer::{Col, sparse::{CreationError, SparseColMat, Triplet}};

use crate::{graph::Graph, parser::{GraphMTX, GraphTSV, ParseError, Parser}};

pub struct Matrix {
    inner: SparseColMat<usize, f64>
}

impl Matrix {
    pub fn new(inner: SparseColMat<usize, f64>) -> Self {
        Self { inner }
    }

    pub fn from_mtx(path: &str) -> Result<(Self, GraphMTX), ParseError> {
        let graph = Parser::parse_mtx(path)?;

        let triplets: Vec<Triplet<usize, usize, f64>> = graph
            .edges
            .iter()
            .map(|&(src, dst)| Triplet::new(dst, src, 1.0))
            .collect();

        let mat = SparseColMat::try_new_from_triplets(
            graph.nrows,
            graph.ncols,
            &triplets,
        ).map_err(ParseError::MatrixError)?;

        Ok((Self::new(mat), graph))
    }

    pub fn from_tsv(path_articles: &str, path_categories: &str, path_links: &str) -> Result<(Self, GraphTSV), ParseError> {
        let graph = Parser::parse_tsv(path_articles, path_categories, path_links)?;

        let triplets: Vec<Triplet<usize, usize, f64>> = graph
            .edges
            .iter()
            .map(|&(src, dst)| Triplet::new(dst, src, 1.0))
            .collect();

        let mat = SparseColMat::try_new_from_triplets(
            graph.nodes.len(),
            graph.nodes.len(),
            &triplets,
        ).map_err(ParseError::MatrixError)?;

        Ok((Self::new(mat), graph))
    }

    pub fn from_graph(graph: &Graph) -> Result<Self, CreationError> {
        let mut triplets = Vec::with_capacity(graph.n_edges());

        for (src, dsts) in graph.adj_list().iter().enumerate() {
            for (dst, weight) in dsts {
                triplets.push(Triplet::new(*dst, src, *weight as f64));
            }
        }

        let mat = SparseColMat::try_new_from_triplets(
            graph.n_nodes(),
            graph.n_nodes(),
            &triplets
        );

        mat.map(Matrix::new)
    }

    const PAGERANK_ALPHA: f64 = 0.85;

    pub fn pagerank(
        self,
        alpha: f64, 
        tol: f64,
        max_iter: Option<usize>
    ) -> (Col<f64>, f64) {
        let mut a = self.inner;

        let n = a.nrows();
        let nf = n as f64;

        let e: Col<f64> = Col::ones(n);

        // 1. Compute Nj: number of outgoing edges per node
        let mut nj = vec![0.0f64; n];
        for (col, count) in nj.iter_mut().enumerate() {
            *count = a.col_range(col).len() as f64;
        }

        // 2. Compute Dj: Indicator of null Nj columns
        let dj = Col::from_fn(n, |i| if nj[i] == 0.0 { 1.0 } else { 0.0 });

        // 3. Normalize A
        for (col, count) in nj.iter().enumerate() {
            if *count != 0.0 {
                let inv = 1.0 / count;

                for val in a.val_of_col_mut(col) {
                    *val = inv;
                }
            }
        }

        // 4. Compute V 
        let v = alpha * dj.transpose() + (1.0 - alpha) * e.transpose();

        // 5. Run the "Power Iteration" method
        let mut r_next;
        let mut r: Col<f64> = Col::zeros(n);
        r[0] = 1.0;

        let mut iter = 0;

        while max_iter.is_none_or(|max_iter| iter < max_iter) {
            r = &r / r.norm_l2();

            let vr = &v * &r;
            r_next = alpha * &a * &r + (&e * vr) / nf;

            if (&r_next - &r).norm_l2() <= tol {
                break;
            }

            std::mem::swap(&mut r, &mut r_next);

            iter += 1;
        }

        // 6. Compute tolerance
        let vr = &v * &r;
        r_next = alpha * &a * &r + (&e * vr) / nf;

        let tol = (r_next - &r).norm_l2();
        
        (r, tol)
    }
} 

impl Graph {
    pub fn conn_matrix(&self) -> Result<Matrix, CreationError> {
        Matrix::from_graph(self)
    }
}

pub struct PagerankBuilder {
    mat: Matrix,
    alpha: f64,
    tol: f64,
    max_iter: Option<usize>,
}

impl PagerankBuilder {
    pub fn new(mat: Matrix) -> Self {
        Self {
            mat,
            alpha: Matrix::PAGERANK_ALPHA,
            tol: 0.0,
            max_iter: None,
        }
    }

    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = alpha.clamp(0.0, 1.0);
        self
    }

    pub fn tolerance(mut self, tol: f64) -> Self {
        self.tol = tol.max(0.0);
        self
    }

    pub fn max_iter(mut self, max_iter: usize) -> Self {
        self.max_iter = Some(max_iter);
        self
    }

    pub fn run(self) -> (Col<f64>, f64) {
        self.mat.pagerank(
            self.alpha,
            self.tol,
            self.max_iter
        )
    }
}
