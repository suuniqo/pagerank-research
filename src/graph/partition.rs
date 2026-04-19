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

    pub fn neighbour_community_weights(graph: &Graph, v: usize, community: &[usize]) -> HashMap<usize, usize> {
        let mut neig_comm_weights = HashMap::new();
        let neigs = &graph.adj_list[v];

        for &(v, weight) in neigs {
                let cv = community[v];
                *neig_comm_weights.entry(cv).or_insert(0) += weight;
        }

        neig_comm_weights
    }

    fn louvain_moves(graph: &'g Graph, resolution: f64, m: f64, gain_treshold: f64) -> Option<Vec<usize>> {
        // Community partition
        let mut community: Vec<usize> = (0..graph.n_nodes).collect();

        // Sum of the degrees of the nodes in each community
        let mut strength_comm = (0..graph.n_nodes)
            .map(|node| graph.strength_unchecked(node))
            .collect::<Vec<usize>>();

        let mut total_gain = 0.0;

        let mut change = true;

        while change {
            change = false;

            let mut order: Vec<usize> = (0..graph.n_nodes).collect();
            order.shuffle(&mut rng());
            
            for &u in &order {
                let neig_comm_weights 
                    = Self::neighbour_community_weights(graph, u, &community);

                let curr_comm = community[u];
                let curr_strength = graph.strength_unchecked(u);

                // exclude u from the degree count
                strength_comm[curr_comm] -= curr_strength;

                let community_delta = |c: usize| -> f64 {
                    let &wt = neig_comm_weights.get(&c).unwrap_or(&0);
                    // As mentioned above this is m times the change in modularity
                    // caused by moving a node out of the community.
                    -(wt as f64) + 0.5 * resolution * (strength_comm[c] as f64) * (curr_strength as f64) / m
                };

                let mut max_gain = 0.0;
                let mut best_comm  = curr_comm;

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

                if best_comm != curr_comm {
                    community[u] = best_comm;

                    total_gain += max_gain;

                    change = true;
                }
            }

            println!("gain: {total_gain}");
        }

        if total_gain < m * gain_treshold {
            return None;
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

        Some(new_community)
    }

    pub fn from_louvain(graph: &'g Graph, resolution: f64, max_iterations: usize, gain_treshold: f64) -> Self {
        let m = graph.weights().sum() as f64;
        
        let mut iter = 0;
        while iter < max_iterations 
            && let Some(community) = Self::louvain_moves(graph, resolution, m, gain_treshold) 
        {
                
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

            a[cu] += self.graph.strength_unchecked(u);

            for &(v, weight) in neig {
                if self.community[v] == cu {
                    e[cu] += weight;
                }
            }
        }

        let mut q = 0.0;

        let m = self.graph.weights().sum::<usize>();

        for c in 0..self.n_partitions {
            q += e[c] as f64 - (a[c].pow(2) as f64 / (2 * m) as f64);
        }

        q / (2 * m) as f64
    }

    pub fn len(&self) -> usize {
        self.n_partitions
    }
}
