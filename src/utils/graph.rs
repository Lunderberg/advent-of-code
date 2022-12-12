use std::cmp::Reverse;
use std::collections::HashMap;
use std::hash::Hash;

use itertools::Itertools;
use priority_queue::PriorityQueue;

#[derive(Debug)]
pub enum Error {
    NoPathToTarget,
    InvalidReverseIndex,
    CircularReversePath,
}

pub trait DynamicGraphNode: Eq + Hash + std::fmt::Debug {}

impl<T> DynamicGraphNode for T where T: Eq + Hash + std::fmt::Debug {}

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
}

#[derive(PartialEq, Eq)]
struct GraphEdge {
    // Index into a vector of nodes, where all elements of that vector
    // have the fastest path known.
    initial_node: usize,
    edge_weight: u64,
    // Can't be set initially, because it can't yet be added to the
    // vector of nodes with known paths. Is there anywhere I can even
    // set this?
    final_node: Option<usize>,
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
                            final_node: None,
                        }),
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
}
