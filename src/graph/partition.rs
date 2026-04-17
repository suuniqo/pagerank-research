use super::Graph;

pub struct PartitionSet<'g> {
    graph: &'g Graph,
    n_partitions: usize,
    tags: Vec<usize>
}

impl<'g> PartitionSet<'g> {
    /// Creates the 'singleton' partition: All nodes in the same community
    pub fn singleton(graph: &'g Graph) -> Self {
        Self {
            graph,
            n_partitions: 1,
            tags: vec![0; graph.n_nodes]
        }
    }

    /// Creates the 'trivial' partition: Each node in its separate partition
    pub fn trivial(graph: &'g Graph) -> Self {
        Self {
            graph,
            n_partitions: graph.n_nodes,
            tags: (0..graph.n_nodes).collect()
        }
    }

    pub fn tag_of_node(&self, v: usize) -> usize {
        self.tags[v]
    }

    /// Computes the modularity of the [PartitionSet]
    pub fn modularity(&self) -> f64 {
        todo!()
    }
}
