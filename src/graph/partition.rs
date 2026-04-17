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
}
