use std::fs::File;
use std::io::{BufWriter, Write};

use crate::graph::partition::PartitionSet;

pub struct Painter;

impl Painter {
    fn hsv_hex(i: usize) -> String {
        let h = (i as f32 * 0.618034) % 1.0;

        let (r, g, b) = Self::hsv_to_rgb(h, 0.65, 0.95);

        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
        let i = (h * 6.0).floor() as i32;
        let f = h * 6.0 - i as f32;

        let p = v * (1.0 - s);
        let q = v * (1.0 - f * s);
        let t = v * (1.0 - (1.0 - f) * s);

        let (r, g, b) = match i % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };

        ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }

    pub fn draw_aggregate(partition: &PartitionSet, output: &str) {
        let mut f = BufWriter::new(File::create(output).unwrap());

        writeln!(f, "graph G {{").unwrap();

        // layout settings
        writeln!(f, "  layout=sfdp;").unwrap();
        writeln!(f, "  overlap=false;").unwrap();
        writeln!(f, "  splines=true;").unwrap();

        // global node style (NO fixedsize)
        writeln!(
            f,
            "  node [style=filled, shape=circle, fixedsize=true, fontsize=10, fontcolor=black];"
        )
        .unwrap();

        // -----------------------------
        // compute sizes
        // -----------------------------
        let comm_sizes = partition
            .communities()
            .into_iter()
            .map(|c| c.len())
            .collect::<Vec<usize>>();

        let max_size = *comm_sizes
            .iter()
            .max()
            .unwrap_or(&1);

        // -----------------------------
        // nodes
        // -----------------------------
        let g = partition.aggregate_graph();

        for (i, size) in comm_sizes.iter().enumerate() {
            let width = 0.3 + 2.5 * (*size as f64 / max_size as f64);
            let label = format!("{}", size);

            let color = Self::hsv_hex(i);

            writeln!(
                f,
                "  {} [label=\"{}\", width={:.3}, fillcolor=\"{}\"]",
                i, label, width, color
            )
            .unwrap();
        }

        // -----------------------------
        // edges
        // -----------------------------
        let mut max_e = 1.0f32;

        for u in 0..g.n_nodes {
            for &(_, w) in &g.adj_list[u] {
                max_e = max_e.max(w as f32);
            }
        }

        for u in 0..g.n_nodes {
            for &(v, w) in &g.adj_list[u] {
                if u >= v {
                    continue;
                }

                let pen = 0.1 + 4.0 * (w as f32 / max_e);

                writeln!(
                    f,
                    "  {} -- {} [penwidth={:.3}]",
                    u, v, pen
                )
                .unwrap();
            }
        }

        writeln!(f, "}}").unwrap();
    }

    pub fn draw_partition(partition: &PartitionSet, output: &str) {
        let g = partition.graph();

        let mut f = BufWriter::new(File::create(output).unwrap());

        writeln!(f, "graph G {{").unwrap();

        // layout settings
        writeln!(f, "  layout=sfdp;").unwrap();
        writeln!(f, "  overlap=false;").unwrap();
        writeln!(f, "  splines=curved;").unwrap();

        // uniform node style (NO fixedsize needed)
        writeln!(
            f,
            "  node [style=filled, shape=circle, fontsize=10, fontcolor=black];"
        )
        .unwrap();

        // -----------------------------
        // nodes (uniform size, community color only)
        // -----------------------------
        for i in 0..g.n_nodes {
            let color = Self::hsv_hex(partition.community(i));

            writeln!(
                f,
                "  {} [width=0.5, fixedsize=true, fillcolor=\"{}\"]",
                i, color
            )
            .unwrap();
        }

        // -----------------------------
        // edges (weighted)
        // -----------------------------
        let mut max_e = 1.0f32;

        for u in 0..g.n_nodes {
            for &(_, w) in &g.adj_list[u] {
                max_e = max_e.max(w as f32);
            }
        }

        for u in 0..g.n_nodes {
            for &(v, w) in &g.adj_list[u] {
                if u >= v {
                    continue;
                }

                let pen = 0.1 + 4.0 * (w as f32 / max_e);

                writeln!(
                    f,
                    "  {} -- {} [penwidth={:.3}]",
                    u, v, pen
                )
                .unwrap();
            }
        }

        writeln!(f, "}}").unwrap();
    }
}
