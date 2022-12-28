#![allow(unused_imports)]
use crate::utils::extensions::TakeWhileInclusive;
use crate::utils::graph::DynamicGraph;
use crate::{Error, Puzzle};

use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use bit_set::BitSet;
use itertools::{Either, Itertools};

#[derive(Debug)]
pub struct ValveSystem {
    valves: Vec<Valve>,
    cached_path_lengths: HashMap<(usize, usize), u64>,
}

#[derive(Debug)]
struct Valve {
    name: String,
    index: usize,
    flow_rate: u64,
    tunnels: Vec<usize>,
}

struct ValveSpec {
    name: String,
    flow_rate: u64,
    tunnels: Vec<String>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
enum ActorState {
    Start { location: usize },
    MoveTo { dest: usize, time_to_finish: u64 },
    Idle,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct SearchState {
    time_remaining: u64,
    my_state: ActorState,
    elephant_state: ActorState,
    valves_available: BitSet,
    open_valves: BitSet,
}

#[derive(Debug, Default, Clone)]
struct ActorChoiceResult {
    valves_claimed: BitSet,
    valves_opened: BitSet,
}

struct ActorDisplay<'a> {
    system: &'a ValveSystem,
    actor: &'a ActorState,
}

impl Display for ActorDisplay<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.actor {
            ActorState::Start { location } => {
                write!(f, "Start({})", self.system.valves[*location].name)
            }
            ActorState::MoveTo {
                dest,
                time_to_finish: time_remaining,
            } => write!(
                f,
                "MoveTo({}, t={time_remaining})",
                self.system.valves[*dest].name
            ),
            ActorState::Idle => write!(f, "Idle"),
        }
    }
}

impl ValveSystem {
    fn max_flow_rate(&self) -> u64 {
        self.valves.iter().map(|valve| valve.flow_rate).sum()
    }

    fn useful_valves(&self) -> BitSet {
        self.valves
            .iter()
            .enumerate()
            .filter_map(|(i, valve)| (valve.flow_rate > 0).then(|| i))
            .collect()
    }

    fn start_location(&self) -> Option<usize> {
        self.valves
            .iter()
            .find(|valve| valve.name == "AA")
            .map(|valve| valve.index)
    }

    fn generate_path_cache(&mut self) {
        let start_location = self.start_location().unwrap();
        let useful_valves = self.useful_valves();
        self.cached_path_lengths = useful_valves
            .iter()
            .chain(std::iter::once(start_location))
            .flat_map(|from_index| {
                self.dijkstra_paths(from_index)
                    .into_iter()
                    .filter(|(node, _)| useful_valves.contains(*node))
                    .map(move |(node, info)| {
                        ((from_index, node), info.initial_to_node)
                    })
            })
            .collect()
    }

    fn initial_state_part_1(&self) -> Result<SearchState, Error> {
        let location = self.start_location().ok_or(Error::NoStartPosition)?;
        Ok(SearchState {
            time_remaining: 30,
            my_state: ActorState::Start { location },
            elephant_state: ActorState::Idle,
            valves_available: self.useful_valves(),
            open_valves: BitSet::with_capacity(self.valves.len()),
        })
    }
    fn initial_state_part_2(&self) -> Result<SearchState, Error> {
        let location = self.start_location().ok_or(Error::NoStartPosition)?;
        Ok(SearchState {
            time_remaining: 26,
            my_state: ActorState::Start { location },
            elephant_state: ActorState::Start { location },
            valves_available: self.useful_valves(),
            open_valves: BitSet::with_capacity(self.valves.len()),
        })
    }
}

impl SearchState {
    fn current_rate(&self, system: &ValveSystem) -> u64 {
        self.open_valves
            .iter()
            .map(|index| system.valves[index].flow_rate)
            .sum()
    }

    // fn pressure_relief_if_idle(&self, system: &ValveSystem) -> u64 {
    //     self.current_rate(system) * self.time_remaining
    // }

    // fn pressure_relief_of_idealized_path(&self, system: &ValveSystem) -> u64 {
    //     match self.my_state {
    //         ActorState::Start { location } => {
    //             Some((location, self.time_remaining))
    //         }
    //         ActorState::MoveTo {
    //             dest,
    //             time_to_finish,
    //         } => Some((dest, self.time_remaining - time_to_finish)),
    //         ActorState::Idle => None,
    //     }
    //     .map_or(0, |(location, time_remaining)| {
    //         self.valves_available
    //             .difference(&std::iter::once(location).collect())
    //             .map(|index| system.valves[index].flow_rate)
    //             .sorted_by_key(|rate| std::cmp::Reverse(*rate))
    //             .take((time_remaining as usize) / 2)
    //             .enumerate()
    //             .map(|(i, rate)| {
    //                 let t: u64 = self
    //                     .time_remaining
    //                     .checked_sub(2 * (i + 1) as u64)
    //                     .expect("Check sub failure during idealized path");
    //                 t * rate
    //             })
    //             .sum::<u64>()
    //     })
    // }

    // fn pressure_relief_upper_bound(&self, system: &ValveSystem) -> u64 {
    //     self.pressure_relief_if_idle(system)
    //         + self.pressure_relief_of_idealized_path(system)
    // }
}

impl DynamicGraph<SearchState> for ValveSystem {
    fn connections_from(&self, node: &SearchState) -> Vec<(SearchState, u64)> {
        if node.time_remaining == 0 {
            return Vec::new();
        }

        let max_flow_rate = self.max_flow_rate();
        let time_remaining = node.time_remaining.checked_sub(1).unwrap();

        node.my_state
            .next_state_options(
                &node.valves_available,
                &self.cached_path_lengths,
                node.time_remaining,
            )
            .into_iter()
            .flat_map(|(my_state, my_results)| {
                let valves_available: BitSet = node
                    .valves_available
                    .difference(&my_results.valves_claimed)
                    .collect();
                node.elephant_state
                    .next_state_options(
                        &valves_available,
                        &self.cached_path_lengths,
                        node.time_remaining,
                    )
                    .map(move |(elephant_state, elephant_results)| {
                        let valves_claimed: BitSet = my_results
                            .valves_claimed
                            .union(&elephant_results.valves_claimed)
                            .collect();
                        let valves_opened: BitSet = my_results
                            .valves_opened
                            .union(&elephant_results.valves_opened)
                            .collect();
                        (
                            my_state.clone(),
                            elephant_state,
                            ActorChoiceResult {
                                valves_claimed,
                                valves_opened,
                            },
                        )
                    })
            })
            .map(|(my_state, elephant_state, choice_results)| {
                let valves_available: BitSet = node
                    .valves_available
                    .difference(&choice_results.valves_claimed)
                    .collect();
                let open_valves: BitSet = node
                    .open_valves
                    .union(&choice_results.valves_opened)
                    .collect();
                let new_search_state = SearchState {
                    time_remaining,
                    my_state,
                    elephant_state,
                    valves_available,
                    open_valves,
                };
                let missing_flow_rate =
                    max_flow_rate - new_search_state.current_rate(&self);

                (new_search_state, missing_flow_rate)
            })
            .collect()
    }

    fn heuristic_between(
        &self,
        _node_from: &SearchState,
        _node_to: &SearchState,
    ) -> Option<u64> {
        Some(1)
    }
}

impl ActorState {
    fn next_state_options(
        &self,
        valves_available: &BitSet,
        cached_path_lengths: &HashMap<(usize, usize), u64>,
        time_remaining: u64,
    ) -> impl Iterator<Item = (Self, ActorChoiceResult)> {
        match self.clone() {
            ActorState::Idle => Either::Left(std::iter::once((
                ActorState::Idle,
                ActorChoiceResult::default(),
            ))),
            ActorState::MoveTo {
                dest,
                time_to_finish: 0,
            } => Either::Right(Either::Right(
                Self::choose_next_destination(
                    dest,
                    valves_available,
                    cached_path_lengths,
                    time_remaining,
                )
                .map(move |(actor, res)| {
                    let newly_opened: BitSet = std::iter::once(dest).collect();
                    (
                        actor,
                        ActorChoiceResult {
                            valves_claimed: res.valves_claimed,
                            valves_opened: res
                                .valves_opened
                                .union(&newly_opened)
                                .collect(),
                        },
                    )
                }),
            )),
            ActorState::MoveTo {
                dest,
                time_to_finish,
            } => Either::Left(std::iter::once((
                ActorState::MoveTo {
                    dest,
                    time_to_finish: time_to_finish - 1,
                },
                ActorChoiceResult::default(),
            ))),
            ActorState::Start { location } => {
                Either::Right(Either::Left(Self::choose_next_destination(
                    location,
                    valves_available,
                    cached_path_lengths,
                    time_remaining,
                )))
            }
        }
    }

    fn choose_next_destination(
        current_location: usize,
        valves_available: &BitSet,
        cached_path_lengths: &HashMap<(usize, usize), u64>,
        time_remaining: u64,
    ) -> impl Iterator<Item = (Self, ActorChoiceResult)> {
        let options: Vec<(Self, ActorChoiceResult)> = valves_available
            .iter()
            .map(|next_location| {
                let distance = cached_path_lengths
                    .get(&(current_location, next_location))
                    .unwrap();
                // let time = distance + 1;
                let time = *distance;
                (next_location, time)
            })
            .filter(|(_next_location, time)| *time < time_remaining)
            .map(|(next_location, time)| {
                let actor_state = ActorState::MoveTo {
                    dest: next_location,
                    time_to_finish: time,
                };
                let valves_claimed: BitSet =
                    std::iter::once(next_location).collect();
                (
                    actor_state,
                    ActorChoiceResult {
                        valves_claimed,
                        ..Default::default()
                    },
                )
            })
            .collect();

        if options.is_empty() {
            Either::Left(std::iter::once((
                ActorState::Idle,
                ActorChoiceResult::default(),
            )))
        } else {
            Either::Right(options.into_iter())
        }
    }
}

impl std::str::FromStr for ValveSpec {
    type Err = Error;
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let orig_line = line.to_string();
        let words = line.splitn(10, ' ');
        let mut words = words.skip(1); // "Valve"
        let name = words
            .next()
            .ok_or(Error::InvalidString(orig_line.clone()))?
            .to_string();
        let mut words = words.skip(2); // "has flow"
        let flow_rate: u64 = words
            .next()
            .ok_or(Error::InvalidString(orig_line.clone()))?
            .strip_prefix("rate=")
            .and_then(|s| s.strip_suffix(';'))
            .map(|r| r.parse::<u64>())
            .ok_or(Error::InvalidString(orig_line.clone()))??;
        let mut words = words.skip(4); // "tunnels lead to"
        let tunnels: Vec<_> = words
            .next()
            .ok_or(Error::InvalidString(orig_line.clone()))?
            .split(", ")
            .map(|s| s.to_string())
            .collect();
        Ok(ValveSpec {
            name,
            flow_rate,
            tunnels,
        })
    }
}

impl DynamicGraph<usize> for ValveSystem {
    fn connections_from(&self, node: &usize) -> Vec<(usize, u64)> {
        self.valves[*node]
            .tunnels
            .iter()
            .copied()
            .map(|dest| (dest, 1))
            .collect()
    }

    fn heuristic_between(
        &self,
        _node_from: &usize,
        _node_to: &usize,
    ) -> Option<u64> {
        Some(1)
    }
}

pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;
    const YEAR: u32 = 2022;
    const DAY: u8 = 16;

    type ParsedInput = ValveSystem;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let specs = lines
            .map(|line| line.parse::<ValveSpec>())
            .collect::<Result<Vec<_>, _>>()?;
        let name_map: HashMap<_, _> = specs
            .iter()
            .map(|spec| spec.name.clone())
            .enumerate()
            .map(|(i, name)| (name, i))
            .collect();
        let valves = specs
            .into_iter()
            .enumerate()
            .map(|(index, spec)| Valve {
                name: spec.name,
                index,
                flow_rate: spec.flow_rate,
                tunnels: spec
                    .tunnels
                    .into_iter()
                    .map(|name| *name_map.get(&name).unwrap())
                    .collect(),
            })
            .collect();

        let mut system = ValveSystem {
            valves,
            cached_path_lengths: HashMap::new(),
        };
        system.generate_path_cache();

        Ok(system)
    }

    type Part1Result = u64;
    fn part_1(system: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        let initial_state = system.initial_state_part_1()?;
        println!("Initial state: {:?}", initial_state);

        let idealized = initial_state.time_remaining * system.max_flow_rate();
        let orderings: Vec<_> = system
            .dijkstra_search(initial_state)
            .take_while_inclusive(|(node, _)| node.time_remaining > 0)
            .collect();
        let best = orderings
            .iter()
            .filter(|(_, info)| info.num_out_edges == 0)
            .min_by_key(|(_, info)| info.initial_to_node)
            .unwrap();

        let path_str = std::iter::successors(Some(best), |(_, info)| {
            info.backref
                .as_ref()
                .map(|edge| &orderings[edge.initial_node])
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|(node, _)| {
            format!(
                "(t={}, rate={}, opened=[{}], me={})",
                node.time_remaining,
                node.current_rate(&system),
                node.open_valves
                    .iter()
                    .map(|index| &system.valves[index].name)
                    .join(", "),
                ActorDisplay {
                    actor: &node.my_state,
                    system: &system
                },
            )
        })
        .join("\n\t=> ");

        println!("Path for best: {path_str}");

        Ok(idealized - (best.1.initial_to_node as u64))
    }

    type Part2Result = u64;
    fn part_2(system: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        let initial_state = system.initial_state_part_2()?;
        let idealized = initial_state.time_remaining * system.max_flow_rate();
        println!("Initial state: {:?}", initial_state);

        let orderings: Vec<_> = system
            .dijkstra_search(initial_state)
            .take_while_inclusive(|(node, _)| node.time_remaining > 0)
            .enumerate()
            .inspect(|(i, (node, _))| {
                if i % 100000 == 0 {
                    println!("Examined {}, t={}", i, node.time_remaining)
                }
            })
            .map(|(_i, node)| node)
            .collect();
        let best = orderings
            .iter()
            .filter(|(_, info)| info.num_out_edges == 0)
            .min_by_key(|(_, info)| info.initial_to_node)
            .unwrap();

        let path_str = std::iter::successors(Some(best), |(_, info)| {
            info.backref
                .as_ref()
                .map(|edge| &orderings[edge.initial_node])
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|(node, _info)| {
            format!(
                "(t={}, rate={}, opened=[{}], me={}, elephant={})",
                node.time_remaining,
                node.current_rate(&system),
                node.open_valves
                    .iter()
                    .map(|index| &system.valves[index].name)
                    .join(", "),
                ActorDisplay {
                    actor: &node.my_state,
                    system: &system
                },
                ActorDisplay {
                    actor: &node.elephant_state,
                    system: &system
                },
            )
        })
        .join("\n\t=> ");

        println!("Path for best: {path_str}");

        Ok(idealized - (best.1.initial_to_node as u64))
    }
}
