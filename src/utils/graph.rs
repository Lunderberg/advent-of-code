use std::cmp::Reverse;
use std::collections::HashMap;
use std::hash::Hash;

use priority_queue::PriorityQueue;

#[derive(Debug)]
pub enum Error {
    NoPathToTarget,
}

pub trait DynamicGraphNode: Clone + Eq + Hash {}

impl<T> DynamicGraphNode for T where T: Clone + Eq + Hash {}

struct SearchPointInfo<T: DynamicGraphNode> {
    src_to_pos: u64,
    heuristic_to_dest: u64,
    previous_point: Option<(T, u64)>,
    finalized: bool,
}

impl<T: DynamicGraphNode> SearchPointInfo<T> {
    fn priority(&self) -> Reverse<u64> {
        Reverse(self.src_to_pos + self.heuristic_to_dest)
    }
}

pub trait DynamicGraph<T: DynamicGraphNode> {
    // Given a node, return all nodes directly excessible from that
    // node, along with the cost associated with each edge.
    fn connections_from(&self, node: &T) -> Vec<(T, u64)>;

    // Used for A* search.  If no such heuristic can be generated,
    // return 0 to fall back to using Dijkstra's.
    fn heuristic_between(&self, a: &T, b: &T) -> u64;

    // Returns the shortest path from initial to target, along with
    // the cost of each segment of the path.  Includes the target, but
    // does not include the initial point.
    fn shortest_path(
        &self,
        initial: &T,
        target: &T,
    ) -> Result<Vec<(T, u64)>, Error> {
        let get_heuristic =
            |pos: &T| -> u64 { self.heuristic_between(pos, target) };

        let start_info = SearchPointInfo::<T> {
            src_to_pos: 0,
            heuristic_to_dest: get_heuristic(initial),
            previous_point: None,
            finalized: false,
        };

        let mut search_queue = PriorityQueue::new();
        search_queue.push(initial.clone(), start_info.priority());

        let mut pos_info_map = HashMap::new();
        pos_info_map.insert(initial.clone(), start_info);

        while search_queue.len() > 0 {
            let current_pos = search_queue.pop().unwrap().0;
            let current_info = pos_info_map.get_mut(&current_pos).unwrap();
            current_info.finalized = true;

            if &current_pos == target {
                break;
            }

            let src_to_current_pos = current_info.src_to_pos;

            self.connections_from(&current_pos)
                .into_iter()
                .map(|(edge_target, edge_weight): (T,u64)| -> (T, Option<&SearchPointInfo<T>>,u64) {
                    let info = pos_info_map.get(&edge_target);
                    (edge_target, info, edge_weight)
                })
                // Don't re-check edges that point to a node whose
                // shortest path is known.
                .filter(|(_pos, opt_info, _edge_weight)| {
                    opt_info.map_or(true, |info| !info.finalized)
                })
                .filter_map(|(pos, opt_info, edge_weight)| {
                    let src_to_pos =
                        src_to_current_pos + edge_weight;
                    opt_info
                        .map_or(true, |info| src_to_pos < info.src_to_pos)
                        .then(|| (pos, opt_info, src_to_pos, edge_weight))
                })
                .map(|(pos, opt_info, src_to_pos, edge_weight)| {
                    let info: SearchPointInfo<T> = opt_info.map_or_else(
                        || SearchPointInfo::<T> {
                            src_to_pos,
                            previous_point: Some((current_pos.clone(),edge_weight)),
                            heuristic_to_dest: get_heuristic(&pos),
                            finalized: false,
                        },
                        |info| SearchPointInfo::<T> {
                            src_to_pos,
                            previous_point: Some((current_pos.clone(),edge_weight)),
                            heuristic_to_dest: info.heuristic_to_dest,
                            finalized: false,
                        },
                    );
                    (pos, info)
                })
                .collect::<Vec<_>>()
                .into_iter()
                .for_each(|(pos, info)| {
                    search_queue.push_increase(pos.clone(), info.priority());
                    pos_info_map.insert(pos, info);
                });
        }

        pos_info_map
            .contains_key(target)
            .then(|| {
                let mut pos: T = target.clone();
                std::iter::from_fn(move || {
                    pos_info_map.get(&pos).unwrap().previous_point.as_ref().map(
                        |(edge_source, edge_weight)| {
                            let edge_target = std::mem::replace(
                                &mut pos,
                                edge_source.clone(),
                            );
                            (edge_target, *edge_weight)
                        },
                    )
                })
                .collect()
            })
            .ok_or(Error::NoPathToTarget)
    }
}
