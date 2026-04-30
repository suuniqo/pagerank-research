use faer::{Col, sparse::{SparseColMat, Triplet}};

use crate::parser::{GraphMTX, GraphTSV, Parser, ParseError};

pub mod pagerank;

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
        ).map_err(|err| ParseError::MatrixError(err))?;

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
        ).map_err(|err| ParseError::MatrixError(err))?;

        Ok((Self::new(mat), graph))
    }


    pub fn pagerank(
        self,
        alpha: f64, 
        n_iter: usize
    ) -> (Col<f64>, f64) {
        let mut a = self.inner;

        let n = a.nrows();
        let nf = n as f64;

        let e: Col<f64> = Col::ones(n);

        // 1. Compute Nj: number of outgoing edges per node
        let mut nj = vec![0.0f64; n];
        for col in 0..n {
            nj[col] = a.col_range(col).len() as f64;
        }

        // 2. Compute Dj: Indicator of null Nj columns
        let dj = Col::from_fn(n, |i| if nj[i] == 0.0 { 1.0 } else { 0.0 });

        // 3. Normalize A
        for col in 0..n {
            let nj_col = nj[col];

            if nj_col != 0.0 {
                let inv = 1.0 / nj_col;
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

        for _ in 0..n_iter {
            r = &r / r.norm_l2();

            let vr = &v * &r;
            r_next = alpha * &a * &r + (&e * vr) / nf;

            std::mem::swap(&mut r, &mut r_next);
        }

        // 6. Compute precision
        let vr = &v * &r;
        let next_r = alpha * &a * &r + (&e * vr) / nf;
        let precicion = (next_r - &r).norm_l2();
        
        (r, precicion)
    }
} 
