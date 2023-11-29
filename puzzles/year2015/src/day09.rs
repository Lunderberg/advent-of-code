use std::collections::HashMap;

use aoc_utils::prelude::*;
use bit_set::BitSet;

#[derive(Clone, Debug)]
pub struct Route {
    from: String,
    to: String,
    dist: u64,
}

#[derive(Debug)]
struct Network {
    name_to_index: HashMap<String, usize>,
    routes: HashMap<usize, Vec<(usize, u64)>>,
}

impl FromIterator<Route> for Network {
    fn from_iter<T: IntoIterator<Item = Route>>(iter: T) -> Self {
        let mut name_to_index = HashMap::new();
        let mut lookup = |name: &str| -> usize {
            if let Some(index) = name_to_index.get(name) {
                *index
            } else {
                let index = name_to_index.len();
                name_to_index.insert(name.to_string(), index);
                index
            }
        };

        let routes = iter
            .into_iter()
            //.map(|route| (lookup(route.from), (lookup(route.to), route.dist)))
            .flat_map(|route| {
                [
                    (lookup(&route.from), (lookup(&route.to), route.dist)),
                    (lookup(&route.to), (lookup(&route.from), route.dist)),
                ]
                .into_iter()
            })
            .into_group_map();
        Network {
            name_to_index,
            routes,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum State {
    Begin,
    InProgress { visited: BitSet, location: usize },
    Finish,
}

impl DynamicGraph<State> for Network {
    fn connections_from(&self, node: &State) -> Vec<(State, u64)> {
        let next_nodes = match node {
            State::Begin => self
                .name_to_index
                .values()
                .map(|&location| {
                    let visited = [location].into_iter().collect();
                    (State::InProgress { location, visited }, 0)
                })
                .collect(),
            State::InProgress { visited, .. }
                if visited.len() == self.name_to_index.len() =>
            {
                vec![(State::Finish, 0)]
            }
            State::InProgress { visited, location } => self
                .routes
                .get(location)
                .iter()
                .flat_map(|routes| routes.iter())
                .filter(|(next, _)| !visited.contains(*next))
                .map(|&(next, dist)| {
                    let mut visited = visited.clone();
                    visited.insert(next);
                    (
                        State::InProgress {
                            location: next,
                            visited,
                        },
                        dist,
                    )
                })
                .collect(),
            State::Finish => Vec::new(),
        };

        // println!("From {node:?}, can reach {next_nodes:#?}");

        next_nodes
    }

    fn heuristic_between(&self, _: &State, _: &State) -> Option<u64> {
        Some(0)
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<Route>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines
            .map(|line| -> Result<_, Error> {
                let (from, _, to, _, dist) = line
                    .split_ascii_whitespace()
                    .tuples()
                    .exactly_one_or_err()?;
                let from = from.to_string();
                let to = to.to_string();
                let dist = dist.parse()?;
                Ok(Route { from, to, dist })
            })
            .collect()
    }

    fn part_1(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let network: Network = values.iter().cloned().collect();
        // println!("Network: {network:#?}");
        network
            .dijkstra_search(State::Begin)
            .find(|(state, _)| matches!(state, State::Finish))
            .map(|(_, info)| info.initial_to_node)
            .ok_or(Error::NoPathToDest)
    }

    fn part_2(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let max_distance = values.iter().map(|route| route.dist).max().unwrap();

        let network: Network = values
            .iter()
            .map(|route| Route {
                dist: max_distance - route.dist,
                ..route.clone()
            })
            .collect();

        let lost_distance = network
            .dijkstra_search(State::Begin)
            .find(|(state, _)| matches!(state, State::Finish))
            .map(|(_, info)| info.initial_to_node)
            .ok_or(Error::NoPathToDest)?;

        Ok(max_distance * ((network.name_to_index.len() - 1) as u64)
            - lost_distance)
    }
}
