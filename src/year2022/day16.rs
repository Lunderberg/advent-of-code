#![allow(unused_imports)]
use crate::utils::graph::DynamicGraph;
use crate::{Error, Puzzle};

use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use bit_set::BitSet;
use itertools::Itertools;

#[derive(Debug)]
pub struct ValveSystem {
    valves: Vec<Valve>,
    cached_path_lengths: HashMap<(usize, usize), i64>,
}

#[derive(Debug)]
struct Valve {
    name: String,
    index: usize,
    flow_rate: i64,
    tunnels: Vec<usize>,
}

struct ValveSpec {
    name: String,
    flow_rate: i64,
    tunnels: Vec<String>,
}

#[derive(Debug, Hash, Eq, PartialEq)]
struct SearchState {
    time_remaining: u8,
    location: usize,
    valves_to_open: BitSet,
    open_valves: BitSet,
    // pressure_released: i64,
}

impl ValveSystem {
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
                    .filter(|res| useful_valves.contains(res.node))
                    .map(move |res| {
                        ((from_index, res.node), res.distance as i64)
                    })
            })
            .collect()
    }

    fn initial_state(&self) -> Result<SearchState, Error> {
        let location = self.start_location().ok_or(Error::NoStartPosition)?;
        Ok(SearchState {
            time_remaining: 30,
            location,
            valves_to_open: self.useful_valves(),
            open_valves: BitSet::with_capacity(self.valves.len()),
            // pressure_released: 0,
        })
    }
}

impl SearchState {
    fn current_rate(&self, system: &ValveSystem) -> i64 {
        self.open_valves
            .iter()
            .map(|index| system.valves[index].flow_rate)
            .sum()
    }

    fn pressure_relief_if_idle(&self, system: &ValveSystem) -> i64 {
        self.current_rate(system) * (self.time_remaining as i64)
    }

    fn pressure_relief_of_idealized_path(&self, system: &ValveSystem) -> i64 {
        self.valves_to_open
            .iter()
            .map(|index| system.valves[index].flow_rate)
            .sorted_by_key(|rate| std::cmp::Reverse(*rate))
            .take((self.time_remaining as usize) / 2)
            .enumerate()
            .map(|(i, rate)| {
                let t: u8 = self
                    .time_remaining
                    .checked_sub(2 * (i + 1) as u8)
                    .expect("Check sub failure during idealized path");
                (t as i64) * rate
            })
            .sum::<i64>()
    }

    fn pressure_relief_upper_bound(&self, system: &ValveSystem) -> i64 {
        self.pressure_relief_if_idle(system)
            + self.pressure_relief_of_idealized_path(system)
    }
}

impl DynamicGraph<SearchState> for ValveSystem {
    fn connections_from(&self, node: &SearchState) -> Vec<(SearchState, u64)> {
        if node.time_remaining == 0 {
            return Vec::new();
        }

        let current_rate = node.current_rate(&self);
        let ideal = node.pressure_relief_upper_bound(&self);

        // println!("Current node: {node:?}");
        // println!("Ideal pressure relief would be {ideal}, from_open={}, from idealized path = {}", node.pressure_relief_if_idle(self), node.pressure_relief_of_idealized_path(self));
        // println!("Current rate: {}", node.current_rate(&self));

        let output: Vec<_> = node
            .valves_to_open
            .iter()
            .map(|location| {
                let distance = self
                    .cached_path_lengths
                    .get(&(node.location, location))
                    .unwrap();
                let time = (*distance as u8) + 1;
                (location, time)
            })
            .filter(|(_location, time)| *time < node.time_remaining)
            .map(|(location, time)| -> SearchState {
                let time_remaining = node
                    .time_remaining
                    .checked_sub(time)
                    .expect("Checked sub underflow in connections_from()");
                let location_bits: BitSet = std::iter::once(location).collect();
                let valves_to_open =
                    node.valves_to_open.difference(&location_bits).collect();
                let open_valves =
                    node.open_valves.union(&location_bits).collect();
                SearchState {
                    time_remaining,
                    location,
                    valves_to_open,
                    open_valves,
                }
            })
            .map(|next_state| {
                let ideal_from_next =
                    next_state.pressure_relief_upper_bound(&self);
                let relief_during_steps = current_rate
                    * ((node.time_remaining - next_state.time_remaining)
                        as i64);
                let pseudo_dist =
                    ideal - (ideal_from_next + relief_during_steps);
                (next_state, pseudo_dist as u64)
            })
            .collect();

        if output.is_empty() {
            let idle_state = SearchState {
                time_remaining: 0,
                location: node.location,
                valves_to_open: node.valves_to_open.clone(),
                open_valves: node.open_valves.clone(),
            };
            let pseudo_dist =
                ideal - current_rate * (node.time_remaining as i64);
            vec![(idle_state, pseudo_dist as u64)]
        } else {
            output
        }
    }

    fn heuristic_between(
        &self,
        _node_from: &SearchState,
        _node_to: &SearchState,
    ) -> Option<u64> {
        Some(1)
    }
}

impl PartialEq for Valve {
    fn eq(&self, rhs: &Valve) -> bool {
        self.index == rhs.index
    }
}
impl Eq for Valve {}
impl Hash for Valve {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.index.hash(hasher);
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
        let flow_rate: i64 = words
            .next()
            .ok_or(Error::InvalidString(orig_line.clone()))?
            .strip_prefix("rate=")
            .and_then(|s| s.strip_suffix(';'))
            .map(|r| r.parse::<i64>())
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

    type Part1Result = i64;
    fn part_1(system: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        println!("Valve system: {system:?}");

        // let start_location: usize = system.start_location().unwrap();

        // let useful_valves = system.useful_valves();

        // let path_lengths: HashMap<(usize, usize), u64> = useful_valves
        //     .iter()
        //     .chain(std::iter::once(start_location))
        //     .flat_map(|from_index| {
        //         system
        //             .dijkstra_paths(from_index)
        //             .into_iter()
        //             .filter(|res| useful_valves.contains(res.node))
        //             .map(move |res| ((from_index, res.node), res.distance))
        //     })
        //     .collect();

        // println!("Path lengths: {path_lengths:?}");

        println!("Initial state: {:?}", system.initial_state()?);

        let idealized =
            system.initial_state()?.pressure_relief_upper_bound(&system);
        let orderings = system.dijkstra_paths(system.initial_state()?);
        println!("Num. Orderings: {}", orderings.len());
        println!(
            "Num. Terminal Orderings: {}",
            orderings
                .iter()
                .filter(|node| node.num_out_edges == 0)
                .count()
        );
        let best = orderings
            .iter()
            .filter(|node| node.num_out_edges == 0)
            .max_by_key(|node| idealized - (node.distance as i64))
            .unwrap();

        println!("Best: {best:?}");

        let path: Vec<_> = std::iter::successors(Some(best), |node| {
            node.backref
                .as_ref()
                .map(|edge| &orderings[edge.initial_node])
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

        let path_str = path
            .iter()
            .map(|node| {
                format!(
                    "(valve={}, t={}, rate={})",
                    system.valves[node.node.location as usize].name,
                    node.node.time_remaining,
                    node.node.current_rate(&system),
                )
            })
            .join("\n\t=> ");

        println!("Path for best: {path_str}");

        Ok(idealized - (best.distance as i64))
    }

    type Part2Result = ();
    fn part_2(_valves: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        Err(Error::NotYetImplemented)
    }
}
