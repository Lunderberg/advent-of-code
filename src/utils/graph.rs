use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use itertools::Itertools;
use priority_queue::PriorityQueue;

#[derive(Debug)]
pub enum Error {
    NoPathToTarget,
    InvalidReverseIndex,
    CircularReversePath,
}

pub trait DynamicGraphNode: Eq + Hash {}

impl<T> DynamicGraphNode for T where T: Eq + Hash {}

// Internal structure for path-finding.  Implements Ord based on the
// sum of src_to_pos and heuristic_to_dest.
#[derive(Eq)]
struct SearchPointInfo {
    node_index: Option<usize>,
    src_to_pos: u64,
    heuristic_to_dest: u64,
    // The edge that was followed to reach this node, along the
    // fastest path from the initial node.  Only the initial node may
    // have previous_edge: None.
    previous_edge: Option<GraphEdge>,
    num_out_edges: Option<usize>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GraphEdge {
    // Index into a vector of nodes, where all elements of that vector
    // have the fastest path known.
    pub initial_node: usize,
    pub edge_weight: u64,
}

impl PartialEq for SearchPointInfo {
    fn eq(&self, rhs: &Self) -> bool {
        self.priority().eq(&rhs.priority())
    }
}

impl PartialOrd for SearchPointInfo {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        self.priority().partial_cmp(&rhs.priority())
    }
}

impl Ord for SearchPointInfo {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        self.priority().cmp(&rhs.priority())
    }
}

impl SearchPointInfo {
    fn priority(&self) -> Reverse<u64> {
        Reverse(self.src_to_pos + self.heuristic_to_dest)
    }
}

pub enum SearchResult<T> {
    Success { path: Vec<(T, u64)> },
    HeuristicFailsOnStartPoint,
    NoPathToTarget { reachable: Vec<T> },
    OtherError(Error),
}

#[derive(Debug)]
pub struct DijkstraSearchNode<T> {
    pub node: T,
    pub distance: u64,
    pub backref: Option<GraphEdge>,
    pub num_out_edges: usize,
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

    fn shortest_path_search_result(
        &self,
        initial: T,
        target: T,
    ) -> SearchResult<T> {
        let get_heuristic =
            |pos: &T| -> Option<u64> { self.heuristic_between(pos, &target) };

        let mut search_queue: PriorityQueue<T, SearchPointInfo> =
            PriorityQueue::new();

        if let Some(initial_heuristic) = get_heuristic(&initial) {
            let initial_info = SearchPointInfo {
                node_index: None,
                src_to_pos: 0,
                heuristic_to_dest: initial_heuristic,
                previous_edge: None,
                num_out_edges: None,
            };
            search_queue.push(initial, initial_info);
        } else {
            return SearchResult::HeuristicFailsOnStartPoint;
        }

        let mut finalized_nodes: HashMap<T, SearchPointInfo> = HashMap::new();
        let mut found_target = false;

        while !search_queue.is_empty() {
            let (node, mut info) = search_queue.pop().unwrap();

            let src_to_node = info.src_to_pos;
            found_target = node == target;
            let connected_nodes = self.connections_from(&node);

            let node_index = finalized_nodes.len();
            info.node_index = Some(node_index);
            info.num_out_edges = Some(connected_nodes.len());

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
                    let info = SearchPointInfo {
                        node_index: None,
                        src_to_pos: src_to_node + edge_weight,
                        heuristic_to_dest,
                        previous_edge: Some(GraphEdge {
                            initial_node: node_index,
                            edge_weight,
                        }),
                        num_out_edges: None,
                    };
                    (new_node, info)
                })
                .filter(|(node, _info)| !finalized_nodes.contains_key(node))
                .for_each(|(node, info): (T, SearchPointInfo)| {
                    search_queue.push_increase(node, info);
                });
        }

        if !found_target {
            let reachable = finalized_nodes.into_keys().collect();
            return SearchResult::NoPathToTarget { reachable };
        }

        let mut index_lookup: Vec<Option<(T, SearchPointInfo)>> =
            finalized_nodes
                .into_iter()
                .sorted_by_key(|(_node, info)| info.node_index.unwrap())
                .map(|(node, info)| Some((node, info)))
                .collect();

        let last: (T, SearchPointInfo) =
            index_lookup.last_mut().unwrap().take().unwrap();
        let res_path: Result<Vec<_>, _> =
            std::iter::successors(Some(Ok(last)), |res| {
                res.as_ref().ok().and_then(|(_node, info)| {
                    info.previous_edge.as_ref().map(|edge| {
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
                info.previous_edge.map(move |edge| (node, edge.edge_weight))
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
            SearchPointInfo {
                node_index: None,
                src_to_pos: 0,
                heuristic_to_dest: 0,
                previous_edge: None,
                num_out_edges: None,
            },
        ))
        .collect();
        DijkstraSearchIter {
            search_queue,
            finished: HashSet::new(),
            graph: &self,
        }
    }

    // Returns the short
    fn dijkstra_paths(&self, initial: T) -> Vec<DijkstraSearchNode<T>> {
        let mut results: HashMap<T, SearchPointInfo> = HashMap::new();
        let mut search_queue: PriorityQueue<T, SearchPointInfo> =
            PriorityQueue::new();
        search_queue.push(
            initial,
            SearchPointInfo {
                node_index: None,
                src_to_pos: 0,
                heuristic_to_dest: 0,
                previous_edge: None,
                num_out_edges: None,
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
            info.num_out_edges = Some(out_connections.len());

            let src_to_node: u64 = info.src_to_pos;

            out_connections
                .into_iter()
                .filter(|(new_node, _edge_weight)| {
                    !results.contains_key(new_node) && new_node != &node
                })
                .map(|(new_node, edge_weight)| {
                    (
                        new_node,
                        SearchPointInfo {
                            node_index: None,
                            src_to_pos: src_to_node + edge_weight,
                            heuristic_to_dest: 0,
                            previous_edge: Some(GraphEdge {
                                initial_node: node_index,
                                edge_weight,
                            }),
                            num_out_edges: None,
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
            .map(|(node, info)| DijkstraSearchNode {
                node,
                distance: info.src_to_pos,
                backref: info.previous_edge,
                num_out_edges: info.num_out_edges.unwrap(),
            })
            .collect()
    }
}

pub struct DijkstraSearchIter<
    'a,
    T: Eq + Hash + Clone,
    Graph: DynamicGraph<T> + ?Sized,
> {
    search_queue: PriorityQueue<T, SearchPointInfo>,
    finished: HashSet<T>,
    graph: &'a Graph,
}

impl<'a, T: Eq + Hash + Clone, Graph: DynamicGraph<T> + ?Sized> Iterator
    for DijkstraSearchIter<'a, T, Graph>
{
    type Item = DijkstraSearchNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.search_queue.pop().map(|(node, mut info)| {
            let out_connections = self.graph.connections_from(&node);

            let node_index = self.finished.len();
            info.node_index = Some(node_index);
            info.num_out_edges = Some(out_connections.len());

            let src_to_node: u64 = info.src_to_pos;

            let finished = &mut self.finished;
            finished.insert(node.clone());

            let search_queue = &mut self.search_queue;

            out_connections
                .into_iter()
                .filter(|(new_node, _edge_weight)| !finished.contains(new_node))
                .map(|(new_node, edge_weight)| {
                    (
                        new_node,
                        SearchPointInfo {
                            node_index: None,
                            src_to_pos: src_to_node + edge_weight,
                            heuristic_to_dest: 0,
                            previous_edge: Some(GraphEdge {
                                initial_node: node_index,
                                edge_weight,
                            }),
                            num_out_edges: None,
                        },
                    )
                })
                .for_each(|(new_node, info)| {
                    search_queue.push_increase(new_node, info);
                });

            DijkstraSearchNode {
                node,
                distance: info.src_to_pos,
                backref: info.previous_edge,
                num_out_edges: info.num_out_edges.unwrap(),
            }
        })
    }
}
