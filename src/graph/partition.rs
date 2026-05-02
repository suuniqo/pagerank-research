use std::{collections::{HashMap, VecDeque}, vec, borrow::Cow};

use rand::{rng, seq::SliceRandom};

use super::Graph;

pub struct PartitionSet<'g> {
    graph: &'g Graph,
    n_partitions: usize,
    community: Vec<usize>
}

impl<'g> PartitionSet<'g> {
    const LOUVAIN_THRESHOLD: f64 = 1e-5;
    const LOUVAIN_RESOLUTION: f64 = 1.0;

    /// Creates the 'trivial' partition: All nodes in the same community
    pub fn trivial(graph: &'g Graph) -> Self {
        Self {
            graph,
            n_partitions: 1,
            community: vec![0; graph.n_nodes]
        }
    }

    /// Creates the 'singleton' partition: Each node in its separate partition
    pub fn singleton(graph: &'g Graph) -> Self {
        Self {
            graph,
            n_partitions: graph.n_nodes,
            community: (0..graph.n_nodes).collect()
        }
    }

    /// Creates a partition using the louvain method, which tries to maximize modularity
    pub fn from_louvain(
        graph: &'g Graph,
        fast: bool,
        resolution: f64,
        gain_threshold: f64,
        max_iter: Option<usize>
    ) -> Self {
        let mut iter = 0;
        let mut curr = Self::singleton(graph);

        let partition_move = if fast { Self::fast_louvain_moves } else { Self::louvain_moves };

        let m = graph.weights().sum::<usize>() as f64 / 2.0;

        while max_iter.is_none_or(|max_iter| iter < max_iter)
            && let Some(next) = partition_move(&curr, resolution, m, gain_threshold)
        {
            curr = next;

            iter += 1;
        }

        curr
    }

    // /// Creates a partition using the louvain method, which tries to
    // /// maximize modularity, while preserving community connectivity
    // pub fn from_leiden(
    //     graph: &'g Graph,
    //     resolution: f64,
    //     temperature: f64,
    //     gain_threshold: f64,
    //     max_iter: Option<usize>
    // ) -> Self {
    //     let mut iter = 0;
    //     let mut curr = Self::singleton(graph);
    //
    //     let m = graph.weights().sum::<usize>() as f64 / 2.0;
    //
    //     while max_iter.is_none_or(|max_iter| iter < max_iter)
    //         && let Some(next) = Self::fast_louvain_moves(&curr, resolution, m, gain_threshold)
    //     { 
    //         curr = next;
    //
    //         // refine partition
    //         todo!();
    //
    //         iter += 1;
    //     }
    //
    //     curr
    // }

    /// Returns the community `v` belongs to
    pub fn community(&self, v: usize) -> usize {
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

            a[cu] += self.graph.strength(u);

            for &(v, weight) in neig {
                if self.community[v] == cu {
                    e[cu] += weight;
                }
            }
        }

        let mut q = 0.0;

        let m2 = self.graph.weights().sum::<usize>();

        for c in 0..self.n_partitions {
            q += e[c] as f64 - (a[c].pow(2) as f64 / m2 as f64);
        }

        q / m2 as f64
    }

    pub fn len(&self) -> usize {
        self.n_partitions
    }

    pub fn is_empty(&self) -> bool {
        self.n_partitions == 0
    }

    pub fn graph(&self) -> &Graph {
        self.graph
    }

    pub fn aggregate_graph(&self) -> Cow<'g, Graph> {
        if self.n_partitions == self.graph.n_nodes {
            return Cow::Borrowed(self.graph);
        }

        let mut adj_list = vec![HashMap::new(); self.n_partitions];

        // Construct a new graph where:
        //   - Node `n_i` corresponds to the `i`th community in the partition
        //   - Nodes `n_i` and `n_j` have an edge with weight `w`, where `w` is
        //     the sum of all edge weights connecting nodes in `n_i` and `n_j`.
        for (u, adj) in self.graph.adj_list.iter().enumerate() {
            let cu = self.community[u];
            for &(v, weight) in adj {
                let cv = self.community[v];

                adj_list[cu].entry(cv).and_modify(|w| *w += weight).or_insert(weight);
            }
        }

        let adj_list: Vec<Vec<(usize, usize)>> = adj_list
            .into_iter()
            .map(|adj_map| adj_map.into_iter().collect())
            .collect();

        Cow::Owned(Graph::new(adj_list))
    }

    fn neighbour_community_weights(graph: &Graph, u: usize, community: &[usize]) -> HashMap<usize, usize> {
        let mut neig_comm_weights = HashMap::new();
        let neigs = &graph.adj_list[u];

        for &(v, weight) in neigs {
            if u == v {
                continue;
            }

            let cv = community[v];
            *neig_comm_weights.entry(cv).or_insert(0) += weight;
        }

        neig_comm_weights
    }

    fn compress_partition(prev_partition: &Self, community: &[usize]) -> Self {
        let mut new_community = vec![0; prev_partition.graph.n_nodes];
        let mut final_tag = HashMap::new();
        let mut n_partitions = 0;

        for u in 0..prev_partition.graph.n_nodes {
            let prev_comm = community[prev_partition.community[u]];
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
            community: new_community,
            n_partitions,
            graph: prev_partition.graph,
        }
    }

    fn best_community_move(
        strength_comm: &mut [usize],
        neig_comm_weights: HashMap<usize, usize>,
        curr_comm: usize,
        curr_strength: usize,
        resolution: f64,
        m: f64,
    ) -> (usize, f64) {
        let mut max_gain = 0.0;
        let mut best_comm  = curr_comm;

        // exclude u from the degree count
        strength_comm[curr_comm] -= curr_strength;

        let community_delta = |c: usize| -> f64 {
            let wt = *neig_comm_weights.get(&c).unwrap_or(&0) as f64;
            // As mentioned above this is m times the change in modularity
            // caused by moving a node out of the community.
            -wt + 0.5 * resolution * (strength_comm[c] as f64) * (curr_strength as f64) / m
        };

        let remove_cost = community_delta(best_comm);

        for &c in neig_comm_weights.keys() {
            if c == curr_comm {
                continue;
            }

            let gain = remove_cost - community_delta(c);

            if gain > max_gain {
                max_gain = gain;
                best_comm = c;
            }
        }

        // increase the degree of best community
        strength_comm[best_comm] += curr_strength;

        (best_comm, max_gain)
    }

    fn fast_louvain_moves(partition: &Self, resolution: f64, m: f64, gain_threshold: f64) -> Option<Self> {
        // Community partition
        let graph = partition.aggregate_graph();
        let mut community: Vec<usize> = (0..graph.n_nodes).collect();

        // Sum of the degrees of the nodes in each community
        let mut strength_comm = (0..graph.n_nodes)
            .map(|node| graph.strength(node))
            .collect::<Vec<usize>>();

        let mut total_gain = 0.0;

        let mut queued = vec![true; graph.n_nodes];
        let mut active: VecDeque<_> = (0..graph.n_nodes).collect();

        while let Some(u) = active.pop_front() {
            queued[u] = false;

            let neig_comm_weights = Self::neighbour_community_weights(&graph, u, &community);

            let curr_comm = community[u];
            let curr_strength = graph.strength(u);

            let (best_comm, max_gain) = Self::best_community_move(
                &mut strength_comm,
                neig_comm_weights,
                curr_comm,
                curr_strength,
                resolution,
                m
            );

            if best_comm != curr_comm {
                // a move is performed
                community[u] = best_comm;
                total_gain += max_gain;

                for &(neig, _) in graph.neighbours(u) {
                    if community[neig] != best_comm && !queued[neig] {
                        queued[neig] = true;
                        active.push_back(neig);
                    }
                }
            }
        }

        if total_gain <= m * gain_threshold {
            return None;
        }

        // Compute new partition
        Some(Self::compress_partition(partition, &community))
    }

    fn louvain_moves(partition: &Self, resolution: f64, m: f64, gain_treshold: f64) -> Option<Self> {
        // Community partition
        let graph = partition.aggregate_graph();
        let mut community: Vec<usize> = (0..graph.n_nodes).collect();

        // Sum of the degrees of the nodes in each community
        let mut strength_comm = (0..graph.n_nodes)
            .map(|node| graph.strength(node))
            .collect::<Vec<usize>>();

        let mut total_gain = 0.0;

        let mut made_move = true;

        while made_move {
            made_move = false;

            let mut order: Vec<_> = (0..graph.n_nodes).collect();
            order.shuffle(&mut rng());

            for u in order {
                let neig_comm_weights = Self::neighbour_community_weights(&graph, u, &community);

                let curr_comm = community[u];
                let curr_strength = graph.strength(u);

                let (best_comm, max_gain) = Self::best_community_move(
                    &mut strength_comm,
                    neig_comm_weights,
                    curr_comm,
                    curr_strength,
                    resolution,
                    m
                );

                if best_comm != curr_comm {
                    // a move is performed
                    community[u] = best_comm;

                    total_gain += max_gain;

                    made_move = true;
                }
            }
        }

        if total_gain <= m * gain_treshold {
            return None;
        }

        // Compute new partition
        Some(Self::compress_partition(partition, &community))
    }
}

pub struct LouvainBuilder<'g> {
    graph: &'g Graph,
    fast: bool,
    resolution: f64,
    gain_threshold: f64,
    max_iter: Option<usize>,
}

impl<'g> LouvainBuilder<'g> {
    pub fn new(graph: &'g Graph) -> Self {
        Self {
            graph,
            fast: true,
            resolution: PartitionSet::LOUVAIN_RESOLUTION,
            gain_threshold: PartitionSet::LOUVAIN_THRESHOLD,
            max_iter: None,
        }
    }

    pub fn fast(mut self, fast: bool) -> Self {
        self.fast = fast;
        self
    }

    pub fn resolution(mut self, resolution: f64) -> Self {
        self.resolution = resolution.max(0.0);
        self
    }

    pub fn gain_threshold(mut self, gain_threshold: f64) -> Self {
        self.gain_threshold = gain_threshold.max(0.0);
        self
    }

    pub fn max_iter(mut self, max_iter: usize) -> Self {
        self.max_iter = Some(max_iter);
        self
    }

    pub fn run(self) -> PartitionSet<'g> {
        PartitionSet::from_louvain(
            self.graph,
            self.fast,
            self.resolution,
            self.gain_threshold,
            self.max_iter,
        )
    }
}
