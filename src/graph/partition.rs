use std::{collections::HashMap, vec};

use rand::seq::SliceRandom;
use rand::rng;

use super::Graph;

pub struct PartitionSet<'g> {
    graph: &'g Graph,
    n_partitions: usize,
    community: Vec<usize>
}

impl<'g> PartitionSet<'g> {
    /// Creates the 'singleton' partition: All nodes in the same community
    pub fn trivial(graph: &'g Graph) -> Self {
        Self {
            graph,
            n_partitions: 1,
            community: vec![0; graph.n_nodes]
        }
    }

    /// Creates the 'trivial' partition: Each node in its separate partition
    pub fn singleton(graph: &'g Graph) -> Self {
        Self {
            graph,
            n_partitions: graph.n_nodes,
            community: (0..graph.n_nodes).collect()
        }
    }

    pub fn from_louvain(graph: &'g Graph) -> Self {
        // Community partition
        let mut community: Vec<usize> = (0..graph.n_nodes).collect();

        // Sum of the degrees of the nodes in each community
        let mut deg_comm = (0..graph.n_nodes)
            .map(|node| graph.degree_unchecked(node))
            .collect::<Vec<usize>>();

        let mut total_gain = 0.0;

        let m = graph.n_edges;

        let mut change = true;

        while change {
            change = false;

            let mut order: Vec<usize> = (0..graph.n_nodes).collect();
            order.shuffle(&mut rng());
            
            for &u in &order {
                let neigs = &graph.adj_list[u];
                let mut neig_comm_weights = HashMap::new();

                // Compute number of edges to each of the neighboring communities
                for &v in neigs {
                    let cv = community[v];
                    *neig_comm_weights.entry(cv).or_insert(0) += 1;
                }

                let curr_comm = community[u];
                let curr_deg = graph.degree_unchecked(u);

                let curr_comm_deg = deg_comm[curr_comm];
                let curr_comm_weight = *neig_comm_weights
                    .get(&curr_comm)
                    .unwrap_or(&0);

                // exclude u from the degree count
                deg_comm[curr_comm] -= curr_deg;

                let mut max_delta = 0.0;
                let mut best_comm  = curr_comm;

                for &c in neig_comm_weights.keys() {
                    if c == curr_comm {
                        continue;
                    }

                    let gain = (neig_comm_weights[&c] as f64 - curr_comm_weight as f64) / m as f64;
                    let correction = (curr_deg as f64 * (curr_comm_deg as f64 - deg_comm[c] as f64)) / (2.0 * m as f64 * m as f64);

                    let delta = gain + correction;

                    if delta > max_delta {
                        max_delta = delta;
                        best_comm = c;
                    }
                }

                // increase the degree of best community
                deg_comm[best_comm] += curr_deg;

                if best_comm != curr_comm && max_delta > 0.0 {
                    community[u] = best_comm;

                    total_gain += max_delta;

                    change = true;
                }
            }

            println!("gain: {total_gain}");
        }

        // Compute new partition

        let mut new_community = vec![0; graph.n_nodes];
        let mut final_tag = HashMap::new();
        let mut n_partitions = 0;

        for u in 0..graph.n_nodes {
            let prev_comm = community[u];
            let new_comm = match final_tag.get(&prev_comm) {
                Some(new_comm) => *new_comm,
                None => {
                    let new_comm = n_partitions;
                    n_partitions += 1;

                    final_tag.insert(prev_comm, new_comm);

                    new_comm
                },
            };

            new_community[u] = new_comm;
        }

        Self {
            graph,
            n_partitions,
            community: new_community,
        }
    }

    pub fn community_unchecked(&self, v: usize) -> usize {
        self.community[v]
    }

    /// Partitions the vertices according to their tags
    pub fn communities(&self) -> Vec<Vec<usize>> {
        let mut communities = vec![vec![]; self.n_partitions];

        for (u, tag) in self.community.iter().enumerate() {
            communities[*tag].push(u);
        }

        communities
    }

    /// Computes the modularity of the [PartitionSet]
    pub fn modularity(&self) -> f64 {
        let mut e = vec![0; self.n_partitions];
        let mut a = vec![0; self.n_partitions];

        for (u, neig) in self.graph.adj_list.iter().enumerate() {
            let cu = self.community[u];

            a[cu] += self.graph.degree_unchecked(u);

            for &v in neig {
                if self.community[v] == cu {
                    e[cu] += 1;
                }
            }
        }

        let mut q = 0.0;

        for c in 0..self.n_partitions {
            q += e[c] as f64 - (a[c].pow(2) as f64 / (2 * self.graph.n_edges) as f64);
        }

        q / (2 * self.graph.n_edges) as f64
    }

    pub fn len(&self) -> usize {
        self.n_partitions
    }
}
