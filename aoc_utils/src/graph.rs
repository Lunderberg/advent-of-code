use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use itertools::Itertools;
use priority_queue::PriorityQueue;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Target is not reachable")]
    NoPathToTarget,
    #[error("Back-tracking along path found dangling index")]
    InvalidReverseIndex,
    #[error("Back-tracking along path found loop")]
    CircularReversePath,
}

pub trait DynamicGraphNode: Eq + Hash {}

impl<T> DynamicGraphNode for T where T: Eq + Hash {}

// Internal structure for path-finding.  Implements Ord based on the
// sum of src_to_pos and heuristic_to_dest.
#[derive(Eq)]
struct InternalInfo {
    node_index: Option<usize>,
    initial_to_node: u64,
    heuristic: u64,
    // The edge that was followed to reach this node, along the
    // fastest path from the initial node.  Only the initial node may
    // have previous_edge: None.
    backref: Option<GraphEdge>,
    num_out_edges: usize,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GraphEdge {
    // Index into a vector of nodes, where all elements of that vector
    // have the fastest path known.
    pub initial_node: usize,
    pub edge_weight: u64,
}

impl PartialEq for InternalInfo {
    fn eq(&self, rhs: &Self) -> bool {
        self.priority().eq(&rhs.priority())
    }
}

impl PartialOrd for InternalInfo {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        self.priority().partial_cmp(&rhs.priority())
    }
}

impl Ord for InternalInfo {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.priority().cmp(&rhs.priority())
    }
}

impl InternalInfo {
    fn priority(&self) -> (Reverse<u64>, u64) {
        (
            Reverse(self.initial_to_node + self.heuristic),
            self.initial_to_node,
        )
    }
}

pub enum SearchResult<T> {
    Success { path: Vec<(T, u64)> },
    HeuristicFailsOnStartPoint,
    NoPathToTarget { reachable: Vec<T> },
    OtherError(Error),
}

#[derive(Debug, Clone)]
pub struct SearchNodeMetadata {
    pub initial_to_node: u64,
    pub heuristic: u64,
    pub backref: Option<GraphEdge>,
    pub num_out_edges: usize,
}

impl From<InternalInfo> for SearchNodeMetadata {
    fn from(info: InternalInfo) -> Self {
        Self {
            initial_to_node: info.initial_to_node,
            heuristic: info.heuristic,
            backref: info.backref,
            num_out_edges: info.num_out_edges,
        }
    }
}

pub trait DynamicGraph<T: DynamicGraphNode> {
    // Given a node, return all nodes directly excessible from that
    // node, along with the cost associated with each edge.
    fn connections_from(&self, node: &T) -> Vec<(T, u64)>;

    // Used for A* search.  If no such heuristic can be generated,
    // return 0 to fall back to using Dijkstra's.  If None, implies
    // that it's impossible to reach the target node from the
    // specified point.
    fn heuristic_between(&self, node_from: &T, node_to: &T) -> Option<u64>;

    // Returns the shortest path from initial to target, along with
    // the cost of each segment of the path.  Includes the target, but
    // does not include the initial point.
    fn shortest_path(
        &self,
        initial: T,
        target: T,
    ) -> Result<Vec<(T, u64)>, Error> {
        match self.shortest_path_search_result(initial, target) {
            SearchResult::Success { path } => Ok(path),
            SearchResult::HeuristicFailsOnStartPoint => {
                Err(Error::NoPathToTarget)
            }
            SearchResult::NoPathToTarget { .. } => Err(Error::NoPathToTarget),
            SearchResult::OtherError(err) => Err(err),
        }
    }

    fn a_star_search<F>(
        &self,
        initial: T,
        mut heuristic: F,
    ) -> SearchIter<T, Self, F>
    where
        F: FnMut(&T) -> Option<u64>,
        // TODO: Can I remove the clonable requirement?  If the iterable
        // is (T,Info), I need one instance to return, and one instance to
        // keep in the HashSet of visited nodes.  I *think* I could use
        // GAT to lend out a (&T, Info) instead, but would need to fiddle
        // with it.
        T: Clone,
    {
        let mut search_queue = PriorityQueue::new();
        if let Some(heuristic_to_dest) = heuristic(&initial) {
            let info = InternalInfo {
                node_index: None,
                initial_to_node: 0,
                heuristic: heuristic_to_dest,
                backref: None,
                num_out_edges: 0,
            };
            search_queue.push_increase(initial, info);
        }

        SearchIter {
            search_queue,
            finished: HashSet::new(),
            graph: self,
            heuristic,
            node_index: 0,
        }
    }

    fn shortest_path_search_result(
        &self,
        initial: T,
        target: T,
    ) -> SearchResult<T> {
        let get_heuristic =
            |pos: &T| -> Option<u64> { self.heuristic_between(pos, &target) };

        let mut search_queue: PriorityQueue<T, InternalInfo> =
            PriorityQueue::new();

        if let Some(initial_heuristic) = get_heuristic(&initial) {
            let initial_info = InternalInfo {
                node_index: None,
                initial_to_node: 0,
                heuristic: initial_heuristic,
                backref: None,
                num_out_edges: 0,
            };
            search_queue.push(initial, initial_info);
        } else {
            return SearchResult::HeuristicFailsOnStartPoint;
        }

        let mut finalized_nodes: HashMap<T, InternalInfo> = HashMap::new();
        let mut found_target = false;

        while !search_queue.is_empty() {
            let (node, mut info) = search_queue.pop().unwrap();

            let src_to_node = info.initial_to_node;
            found_target = node == target;
            let connected_nodes = self.connections_from(&node);

            let node_index = finalized_nodes.len();
            info.node_index = Some(node_index);
            info.num_out_edges = connected_nodes.len();

            finalized_nodes.insert(node, info);

            if found_target {
                break;
            }

            connected_nodes
                .into_iter()
                .filter_map(|(new_node, edge_weight)| {
                    get_heuristic(&new_node).map(move |heuristic_to_dest| {
                        (new_node, edge_weight, heuristic_to_dest)
                    })
                })
                .map(|(new_node, edge_weight, heuristic_to_dest)| {
                    let info = InternalInfo {
                        node_index: None,
                        initial_to_node: src_to_node + edge_weight,
                        heuristic: heuristic_to_dest,
                        backref: Some(GraphEdge {
                            initial_node: node_index,
                            edge_weight,
                        }),
                        num_out_edges: 0,
                    };
                    (new_node, info)
                })
                .filter(|(node, _info)| !finalized_nodes.contains_key(node))
                .for_each(|(node, info): (T, InternalInfo)| {
                    search_queue.push_increase(node, info);
                });
        }

        if !found_target {
            let reachable = finalized_nodes.into_keys().collect();
            return SearchResult::NoPathToTarget { reachable };
        }

        let mut index_lookup: Vec<Option<(T, InternalInfo)>> = finalized_nodes
            .into_iter()
            .sorted_by_key(|(_node, info)| info.node_index.unwrap())
            .map(|(node, info)| Some((node, info)))
            .collect();

        let last: (T, InternalInfo) =
            index_lookup.last_mut().unwrap().take().unwrap();
        let res_path: Result<Vec<_>, _> =
            std::iter::successors(Some(Ok(last)), |res| {
                res.as_ref().ok().and_then(|(_node, info)| {
                    info.backref.as_ref().map(|edge| {
                        index_lookup
                            .get_mut(edge.initial_node)
                            .ok_or(Error::InvalidReverseIndex)
                            .and_then(|opt| {
                                opt.take().ok_or(Error::CircularReversePath)
                            })
                    })
                })
            })
            .filter_map_ok(|(node, info)| {
                info.backref.map(move |edge| (node, edge.edge_weight))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();

        match res_path {
            Ok(path) => SearchResult::Success { path },
            Err(err) => SearchResult::OtherError(err),
        }
    }

    fn dijkstra_search(&self, initial: T) -> DijkstraSearchIter<T, Self>
    where
        T: Clone,
    {
        let search_queue = std::iter::once((
            initial,
            InternalInfo {
                node_index: None,
                initial_to_node: 0,
                heuristic: 0,
                backref: None,
                num_out_edges: 0,
            },
        ))
        .collect();
        DijkstraSearchIter {
            search_queue,
            finished: HashSet::new(),
            graph: self,
        }
    }

    // Returns the short
    fn dijkstra_paths(&self, initial: T) -> Vec<(T, SearchNodeMetadata)> {
        let mut results: HashMap<T, InternalInfo> = HashMap::new();
        let mut search_queue: PriorityQueue<T, InternalInfo> =
            PriorityQueue::new();
        search_queue.push(
            initial,
            InternalInfo {
                node_index: None,
                initial_to_node: 0,
                heuristic: 0,
                backref: None,
                num_out_edges: 0,
            },
        );

        let mut num_processed = 0;

        while !search_queue.is_empty() {
            num_processed += 1;
            if num_processed % 10000 == 0 {
                println!(
                    "Processed {num_processed} nodes, queue size = {}",
                    search_queue.len()
                );
            }

            let (node, mut info) = search_queue.pop().unwrap();
            let out_connections = self.connections_from(&node);

            let node_index = results.len();
            info.node_index = Some(node_index);
            info.num_out_edges = out_connections.len();

            let src_to_node: u64 = info.initial_to_node;

            out_connections
                .into_iter()
                .filter(|(new_node, _edge_weight)| {
                    !results.contains_key(new_node) && new_node != &node
                })
                .map(|(new_node, edge_weight)| {
                    (
                        new_node,
                        InternalInfo {
                            node_index: None,
                            initial_to_node: src_to_node + edge_weight,
                            heuristic: 0,
                            backref: Some(GraphEdge {
                                initial_node: node_index,
                                edge_weight,
                            }),
                            num_out_edges: 0,
                        },
                    )
                })
                .for_each(|(new_node, info)| {
                    search_queue.push_increase(new_node, info);
                });

            results.insert(node, info);
        }

        results
            .into_iter()
            .sorted_by_key(|(_node, info)| info.node_index.unwrap())
            .map(|(node, info)| {
                (
                    node,
                    SearchNodeMetadata {
                        initial_to_node: info.initial_to_node,
                        heuristic: info.heuristic,
                        backref: info.backref,
                        num_out_edges: info.num_out_edges,
                    },
                )
            })
            .collect()
    }
}

pub struct DijkstraSearchIter<
    'a,
    T: Eq + Hash + Clone,
    Graph: DynamicGraph<T> + ?Sized,
> {
    search_queue: PriorityQueue<T, InternalInfo>,
    finished: HashSet<T>,
    graph: &'a Graph,
}

impl<'a, T: Eq + Hash + Clone, Graph: DynamicGraph<T> + ?Sized> Iterator
    for DijkstraSearchIter<'a, T, Graph>
{
    type Item = (T, SearchNodeMetadata);

    fn next(&mut self) -> Option<Self::Item> {
        self.search_queue.pop().map(|(node, mut info)| {
            let out_connections = self.graph.connections_from(&node);

            let node_index = self.finished.len();
            info.node_index = Some(node_index);
            info.num_out_edges = out_connections.len();

            let src_to_node: u64 = info.initial_to_node;

            let finished = &mut self.finished;
            finished.insert(node.clone());

            let search_queue = &mut self.search_queue;

            out_connections
                .into_iter()
                .filter(|(new_node, _edge_weight)| !finished.contains(new_node))
                .map(|(new_node, edge_weight)| {
                    (
                        new_node,
                        InternalInfo {
                            node_index: None,
                            initial_to_node: src_to_node + edge_weight,
                            heuristic: 0,
                            backref: Some(GraphEdge {
                                initial_node: node_index,
                                edge_weight,
                            }),
                            num_out_edges: 0,
                        },
                    )
                })
                .for_each(|(new_node, info)| {
                    search_queue.push_increase(new_node, info);
                });

            let metadata = SearchNodeMetadata {
                initial_to_node: info.initial_to_node,
                heuristic: info.heuristic,
                backref: info.backref,
                num_out_edges: info.num_out_edges,
            };
            (node, metadata)
        })
    }
}

pub struct SearchIter<
    'a,
    T: Eq + Hash + Clone,
    Graph: DynamicGraph<T> + ?Sized,
    F: FnMut(&T) -> Option<u64>,
> {
    search_queue: PriorityQueue<T, InternalInfo>,
    finished: HashSet<T>,
    graph: &'a Graph,
    heuristic: F,
    node_index: usize,
}

impl<
        'a,
        T: Eq + Hash + Clone,
        Graph: DynamicGraph<T> + ?Sized,
        F: FnMut(&T) -> Option<u64>,
    > Iterator for SearchIter<'a, T, Graph, F>
{
    type Item = (T, SearchNodeMetadata);

    fn next(&mut self) -> Option<Self::Item> {
        let (node, mut info) = self.search_queue.pop()?;

        let node_index = self.node_index;
        self.node_index += 1;

        let initial_to_node = info.initial_to_node;
        let heuristic = &mut self.heuristic;
        let finished = &mut self.finished;
        let search_queue = &mut self.search_queue;

        self.graph
            .connections_from(&node)
            .into_iter()
            .inspect(|_| {
                info.num_out_edges += 1;
            })
            .filter(|(new_node, _)| !finished.contains(new_node))
            .filter_map(|(new_node, edge_weight)| {
                heuristic(&new_node).map(move |heuristic_to_dest| {
                    let new_info = InternalInfo {
                        node_index: None,
                        initial_to_node: initial_to_node + edge_weight,
                        heuristic: heuristic_to_dest,
                        backref: Some(GraphEdge {
                            initial_node: node_index,
                            edge_weight,
                        }),
                        num_out_edges: 0,
                    };
                    (new_node, new_info)
                })
            })
            .for_each(|(node, info)| {
                search_queue.push_increase(node, info);
            });

        Some((node, info.into()))
    }
}
