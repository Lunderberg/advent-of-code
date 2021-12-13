#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use itertools::Itertools;

pub struct Day12;

#[derive(Debug)]
struct CaveSystem {
    caves: Vec<Cave>,
    connections: HashMap<usize, Vec<usize>>,
    start: usize,
    end: usize,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum Cave {
    Start,
    End,
    Big(String),
    Small(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TraversalState {
    pos: usize,
    visits_remaining: Vec<VisitsRemaining>,
    free_revisit_available: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum VisitsRemaining {
    Limited(u8),
    LimitedOrOneFreebie(u8),
    Unlimited,
}

impl FromStr for Cave {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "start" {
            Ok(Cave::Start)
        } else if s == "end" {
            Ok(Cave::End)
        } else if s == s.to_ascii_lowercase() {
            Ok(Cave::Small(s.to_string()))
        } else if s == s.to_ascii_uppercase() {
            Ok(Cave::Big(s.to_string()))
        } else {
            Err(Error::InvalidString(s.to_string()))
        }
    }
}

impl Display for Cave {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Cave::*;
        match self {
            Start => write!(f, "start"),
            End => write!(f, "end"),
            Big(s) => write!(f, "{}", s),
            Small(s) => write!(f, "{}", s),
        }
    }
}

impl VisitsRemaining {
    fn after_reduce(&self) -> Self {
        use VisitsRemaining::*;
        match self {
            Limited(val) => Limited(val - 1),
            LimitedOrOneFreebie(val) => LimitedOrOneFreebie(val - 1),
            Unlimited => Unlimited,
        }
    }

    fn can_visit(&self, free_revisit_available: bool) -> bool {
        use VisitsRemaining::*;
        match self {
            Limited(val) => *val > 0,
            LimitedOrOneFreebie(val) => (*val > 0) || free_revisit_available,
            Unlimited => true,
        }
    }
}

impl Cave {
    fn num_visits_allowed(&self) -> VisitsRemaining {
        use Cave::*;
        match self {
            Start => VisitsRemaining::Limited(0),
            End => VisitsRemaining::Limited(1),
            Big(_) => VisitsRemaining::Unlimited,
            Small(_) => VisitsRemaining::LimitedOrOneFreebie(1),
        }
    }
}

impl CaveSystem {
    fn parse<'a>(lines: impl Iterator<Item = &'a str>) -> Result<Self, Error> {
        let all_caves = lines
            .flat_map(|s| s.split('-'))
            .map(|s| s.parse::<Cave>())
            .collect::<Result<Vec<_>, _>>()?;

        let caves: Vec<Cave> = all_caves.iter().unique().cloned().collect();

        let cave_map: HashMap<Cave, usize> = caves
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, cave)| (cave, i))
            .collect();

        let connections = all_caves
            .into_iter()
            .map(|cave| cave_map.get(&cave).unwrap())
            .copied()
            .tuples()
            .flat_map(|(a, b)| vec![(a, b), (b, a)].into_iter())
            .into_group_map();

        let start = *cave_map.get(&Cave::Start).ok_or(Error::NoStartCave)?;
        let end = *cave_map.get(&Cave::End).ok_or(Error::NoEndCave)?;

        Ok(Self {
            caves,
            connections,
            start,
            end,
        })
    }

    fn traversal_graph(
        &self,
        free_revisit_available: bool,
    ) -> Result<(Vec<TraversalState>, Vec<Vec<usize>>), Error> {
        let initial_state = TraversalState {
            pos: self.start,
            visits_remaining: self
                .caves
                .iter()
                .map(|cave| cave.num_visits_allowed())
                .collect(),
            free_revisit_available,
        };

        // First, generate all reachable states and forward edges,
        // each with a unique index for each state.
        let mut initial_indexing: HashMap<TraversalState, usize> =
            HashMap::new();
        initial_indexing.insert(initial_state.clone(), 0);
        let mut to_visit: Vec<TraversalState> = vec![initial_state];
        let mut initial_edge_map: HashMap<usize, Vec<usize>> = HashMap::new();

        while to_visit.len() > 0 {
            let state = to_visit.pop().unwrap();

            let state_num = initial_indexing.get(&state).copied().unwrap();

            let dest_state_nums = state
                .next_states(self)
                .map(|new_state| {
                    initial_indexing
                        .get(&new_state)
                        .copied()
                        .or_else(|| {
                            let state_num = initial_indexing.len();
                            to_visit.push(new_state.clone());
                            initial_indexing.insert(new_state, state_num);
                            Some(state_num)
                        })
                        .unwrap()
                })
                .collect();

            initial_edge_map.insert(state_num, dest_state_nums);
        }

        // Break apart the mappings into vectors, where the
        let initial_nodes: Vec<TraversalState> = initial_indexing
            .into_iter()
            .sorted_by_key(|(_state, i)| *i)
            .map(|(state, _i)| state)
            .collect();
        let initial_forward_edges: Vec<Vec<usize>> = initial_edge_map
            .into_iter()
            .sorted_by_key(|(i, _edges)| *i)
            .map(|(_i, edges)| edges)
            .collect();

        // Sort the output into topological order
        let topological_order = Self::topological_sort(&initial_forward_edges)?;

        let initial_to_topological: Vec<usize> = topological_order
            .iter()
            .enumerate()
            .sorted_by_key(|(_topological_index, initial_index)| *initial_index)
            .map(|(topological_index, _initial_index)| topological_index)
            .collect();

        let ordered_nodes: Vec<TraversalState> = topological_order
            .iter()
            .copied()
            .map(|initial_index_from| initial_nodes[initial_index_from].clone())
            .collect();

        let ordered_forward_edges: Vec<Vec<usize>> = topological_order
            .into_iter()
            .map(|initial_index_from| {
                initial_forward_edges[initial_index_from]
                    .iter()
                    .map(|&initial_index_to| -> usize {
                        initial_to_topological[initial_index_to]
                    })
                    .collect()
            })
            .collect();

        // ordered_nodes
        //     .iter()
        //     .zip(ordered_forward_edges.iter())
        //     .enumerate()
        //     .for_each(|(state_num, (state,states_to))| {
        //         let remaining = state.visits_remaining
        //             .iter()
        //             .enumerate()
        //             .filter_map(|(i,rem)|
        //                 match rem {
        //                     VisitsRemaining::Limited(val) => Some((i,val)),
        //                     VisitsRemaining::LimitedOrOneFreebie(val) => Some((i,val)),
        //                     VisitsRemaining::Unlimited => None,
        //                 }
        //             )
        //             .map(|(i,rem)| format!("{} {}", rem, self.caves[i]))
        //             .join(", ");
        //         let destinations  = states_to
        //             .iter()
        //             .map(|&i|
        //                  format!("{} (#{})", self.caves[ordered_nodes[i].pos], i)
        //             ).join(", ");

        //         println!("State #{}, in cave {}, visits remaining [{}], destinations [{}]",
        //                  state_num,
        //                  self.caves[state.pos],
        //                  remaining,
        //                  destinations);
        //     });

        let is_valid_sort = ordered_forward_edges
            .iter()
            .enumerate()
            .flat_map(|(index_from, indices_to)| {
                indices_to
                    .iter()
                    .copied()
                    .map(move |index_to| (index_from, index_to))
            })
            .all(|(index_from, index_to)| index_from < index_to);
        if !is_valid_sort {
            panic!("This shouldn't ever happen");
        }

        Ok((ordered_nodes, ordered_forward_edges))
    }

    fn topological_sort(
        forward_edges: &Vec<Vec<usize>>,
    ) -> Result<Vec<usize>, Error> {
        let forward_edges: Vec<HashSet<usize>> = forward_edges
            .iter()
            .map(|edges| edges.iter().copied().collect())
            .collect();

        let mut reverse_edges: Vec<HashSet<usize>> =
            forward_edges.iter().map(|_| HashSet::new()).collect();
        forward_edges
            .iter()
            .enumerate()
            .map(|(index_from, indices_to)| {
                indices_to
                    .iter()
                    .map(move |index_to| (index_from, index_to))
            })
            .flatten()
            .for_each(|(index_from, index_to)| {
                reverse_edges[*index_to].insert(index_from);
            });

        let mut ordered_indices: Vec<usize> = reverse_edges
            .iter()
            .enumerate()
            .filter_map(|(i, incoming)| (incoming.len() == 0).then(|| i))
            .collect();
        let mut currently_processing = 0;

        while currently_processing < ordered_indices.len() {
            let pruned_node = ordered_indices[currently_processing];
            forward_edges[pruned_node]
                .iter()
                .filter(|index_to| {
                    reverse_edges[**index_to].remove(&pruned_node);
                    reverse_edges[**index_to].len() == 0
                })
                .for_each(|index_to| ordered_indices.push(*index_to));

            currently_processing += 1;
        }

        if ordered_indices.len() == forward_edges.len() {
            Ok(ordered_indices)
        } else {
            Err(Error::GraphHasCycle)
        }
    }

    fn _enumerate_paths(
        &self,
        free_revisit_available: bool,
    ) -> Result<(), Error> {
        let (states, connections) =
            self.traversal_graph(free_revisit_available)?;

        let mut unfinished: Vec<Vec<usize>> = vec![vec![0]];
        let mut finished: Vec<Vec<usize>> = Vec::new();

        while unfinished.len() > 0 {
            let path = unfinished.pop().unwrap();
            let state = *path.last().unwrap();
            connections[state]
                .iter()
                .map(|next_state| {
                    path.iter()
                        .chain(std::iter::once(next_state))
                        .copied()
                        .collect::<Vec<usize>>()
                })
                .for_each(|new_path| {
                    if states[*new_path.last().unwrap()].is_finished(self) {
                        finished.push(new_path)
                    } else {
                        unfinished.push(new_path)
                    }
                })
        }

        println!("Enumerated {} paths", finished.len());

        finished
            .iter()
            .rev()
            .map(|path| {
                path.iter()
                    .map(|&i| format!("{}", self.caves[states[i].pos]))
                    .join(",")
            })
            .for_each(|line| println!("{}", line));

        Ok(())
    }

    fn num_paths(&self, free_revisit_available: bool) -> Result<usize, Error> {
        let (states, connections) =
            self.traversal_graph(free_revisit_available)?;

        let mut num_paths_to: Vec<usize> = states.iter().map(|_| 0).collect();
        num_paths_to[0] = 1;
        connections
            .iter()
            .enumerate()
            .flat_map(|(index_from, indices_to)| {
                indices_to
                    .iter()
                    .copied()
                    .map(move |index_to| (index_from, index_to))
            })
            .for_each(|(index_from, index_to)| {
                num_paths_to[index_to] += num_paths_to[index_from]
            });

        // states
        //     .iter()
        //     .zip(connections.iter())
        //     .zip(num_paths_to.iter())
        //     .enumerate()
        //     .for_each(|(state_num, ((state,states_to),num_paths))| {
        //         let remaining = state.visits_remaining
        //             .iter()
        //             .enumerate()
        //             .filter_map(|(i,rem)|
        //                 match rem {
        //                     VisitsRemaining::Limited(val) => Some((i,val)),
        //                     VisitsRemaining::LimitedOrOneFreebie(val) => Some((i,val)),
        //                     VisitsRemaining::Unlimited => None,
        //                 }
        //             )
        //             .map(|(i,rem)| format!("{} {}", rem, self.caves[i]))
        //             .join(", ");
        //         let destinations  = states_to
        //             .iter()
        //             .map(|&i|
        //                  format!("{} (#{})", self.caves[states[i].pos], i)
        //             ).join(", ");

        //         println!("{} paths to State #{}, in cave {}, visits remaining [{}], destinations [{}]",
        //                  num_paths,
        //                  state_num,
        //                  self.caves[state.pos],
        //                  remaining,
        //                  destinations);
        //     });

        let total_paths = states
            .iter()
            .zip(num_paths_to.iter())
            .filter(|(state, _num_paths)| state.pos == self.end)
            .map(|(_state, num_paths)| num_paths)
            .sum::<usize>();

        Ok(total_paths)
    }
}

impl TraversalState {
    fn next_states<'a, 'b, 'c>(
        &'a self,
        system: &'b CaveSystem,
    ) -> impl Iterator<Item = Self> + 'c
    where
        'a: 'c,
        'b: 'c,
    {
        let is_finished = self.is_finished(system);
        system
            .connections
            .get(&self.pos)
            .unwrap()
            .clone()
            .into_iter()
            .filter(move |_| !is_finished)
            .flat_map(move |move_to| self.next_state(move_to))
    }

    fn is_finished(&self, system: &CaveSystem) -> bool {
        system.caves[self.pos] == Cave::End
    }

    fn next_state(&self, move_to: usize) -> Option<TraversalState> {
        if self.visits_remaining[move_to].can_visit(false) {
            let visits_remaining = self
                .visits_remaining
                .iter()
                .copied()
                .enumerate()
                .map(|(i, visits_remaining)| {
                    if i == move_to {
                        self.visits_remaining[move_to].after_reduce()
                    } else {
                        visits_remaining
                    }
                })
                .collect();
            Some(Self {
                visits_remaining,
                pos: move_to,
                free_revisit_available: self.free_revisit_available,
            })
        } else if self.visits_remaining[move_to]
            .can_visit(self.free_revisit_available)
        {
            Some(Self {
                visits_remaining: self.visits_remaining.clone(),
                pos: move_to,
                free_revisit_available: false,
            })
        } else {
            None
        }
    }
}

impl Day12 {
    fn parse_caves(&self) -> Result<CaveSystem, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        Ok(CaveSystem::parse(puzzle_input.lines())?)
    }
}

impl Puzzle for Day12 {
    fn day(&self) -> i32 {
        12
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let result = self.parse_caves()?.num_paths(false)?;
        // self.parse_caves()?.enumerate_paths(false)?;
        // let result = ();
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let result = self.parse_caves()?.num_paths(true)?;
        // self.parse_caves()?.enumerate_paths(true)?;
        // let result = ();
        Ok(Box::new(result))
    }
}
