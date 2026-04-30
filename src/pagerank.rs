use faer::sparse::{SparseColMat, SparseColMatRef};
use faer::{Col, Row};

fn clone_matrix(mat: SparseColMatRef<usize, f64>) -> SparseColMat<usize, f64> {
    let val = mat.val();
    let sym = mat.symbolic().to_owned()
        .expect("No errors on cloning");
    SparseColMat::new(sym, val.to_vec())
}

pub fn compute_pagerank(
    conn_mat: SparseColMatRef<usize, f64>, 
    alpha: f64, 
    n_iter: usize) -> (Col<f64>, f64)
{
    let n = conn_mat.nrows();

    let e: Col<f64> = Col::ones(n);
    let e_t = e.transpose();

    // 1. Compute A
    let mut n_j: Row<f64> = Row::zeros(n);
    for elems in conn_mat.triplet_iter() {
        if *elems.val == 0.0 {
            continue;
        }
        n_j[elems.col] += 1.0;
    }

    // dj[j] == 1 <=> Nj[j] != 0
    let dj = n_j.iter()
        .map(|x| if *x == 0.0 { 0.0 } else { 1.0 })
        .collect::<Col<f64>>();

    let mut a = clone_matrix(conn_mat);
    for (col, nj_col) in n_j.iter().enumerate().filter(|x| x.1 != &0.0) {
        for val in a.val_of_col_mut(col) {
            *val = 1.0 / *nj_col;
        }
    }

    // 2. Run the "Power Iteration" method
    let v = alpha * dj.transpose() + (1.0 - alpha) * e_t;

    let mut r: Col<f64> = Col::zeros(n);
    r[0] = 1.0;

    let n = n as f64;
    for _ in 0..n_iter {
        r = &r / r.norm_l2();
        r = alpha * &a * &r + (1.0 / n) * &e * (&v * r);
    }

    let next_r = alpha * &a * &r + (1.0 / n) * &e * (&v * &r);
    let precicion = (next_r - &r).norm_l2();
    
    (r, precicion)
}
