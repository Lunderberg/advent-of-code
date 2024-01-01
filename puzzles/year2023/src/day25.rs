use std::{collections::HashMap, str::FromStr};

use aoc_utils::prelude::*;
use bit_set::BitSet;
use indicatif::ProgressIterator as _;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

#[derive(Debug, Clone)]
pub struct NamedConnection {
    name: String,
    connected_to: Vec<String>,
}

#[derive(Debug)]
struct IndexedGraph {
    connections: Vec<Vec<usize>>,
}

struct TrimmedGraph<'a> {
    connections: &'a Vec<Vec<usize>>,
    removed: [(usize, usize); 3],
}

impl FromStr for NamedConnection {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let (name, connected_to) = line
            .split(':')
            .collect_tuple()
            .ok_or(Error::WrongIteratorSize)?;
        let name = name.to_string();
        let connected_to = connected_to
            .split_ascii_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        Ok(Self { name, connected_to })
    }
}

impl From<Vec<NamedConnection>> for IndexedGraph {
    fn from(connections: Vec<NamedConnection>) -> Self {
        let names: Vec<String> = connections
            .iter()
            .map(|conn| &conn.name)
            .chain(connections.iter().flat_map(|conn| &conn.connected_to))
            .unique()
            .cloned()
            .collect();

        let name_lookup: HashMap<_, _> = names
            .iter()
            .enumerate()
            .map(|(i, name)| (name, i))
            .collect();

        let connections = connections
            .iter()
            .enumerate()
            .flat_map(|(i, conn)| {
                conn.connected_to
                    .iter()
                    .map(|name| {
                        name_lookup
                            .get(name)
                            .cloned()
                            .expect("Could not find index of connection")
                    })
                    .flat_map(move |j| [(i, j), (j, i)])
            })
            .sorted()
            .into_group_map()
            .into_iter()
            .sorted_by_key(|(a, _)| *a)
            .map(|(_, b)| b)
            .collect();

        Self { connections }
    }
}

impl DirectedGraph<usize> for TrimmedGraph<'_> {
    fn connections_from<'a>(
        &'a self,
        from: &'a usize,
    ) -> impl Iterator<Item = usize> + '_ {
        self.connections[*from].iter().copied().filter(move |to| {
            self.removed
                .iter()
                .flat_map(|(i, j)| [(i, j), (j, i)])
                .all(|(i, j)| to != i || from != j)
        })
    }
}

impl IndexedGraph {
    fn segment_sizes<'a>(
        &'a self,
        removed: [(usize, usize); 3],
    ) -> impl Iterator<Item = usize> + 'a {
        let trimmed = TrimmedGraph {
            connections: &self.connections,
            removed,
        };

        let mut to_visit: BitSet = (0..self.connections.len()).collect();

        std::iter::from_fn(move || {
            to_visit.iter().next().map(|starting_point| {
                trimmed
                    .iter_depth_first([starting_point])
                    .inspect(|&index| {
                        to_visit.remove(index);
                    })
                    .count()
            })
        })
    }

    // From https://en.wikipedia.org/wiki/Stoer%E2%80%93Wagner_algorithm
    //
    // TODO: Clean this up.  This is very C-ish code, and not very
    // readable at that.
    fn stoer_wagner_min_cut(&self) -> (i64, Vec<i64>) {
        let n = self.connections.len();
        let mut mat: Vec<Vec<i64>> = vec![vec![0; n]; n];
        self.connections
            .iter()
            .enumerate()
            .flat_map(|(i, outputs)| outputs.iter().map(move |j| (i, *j)))
            .for_each(|(i, j)| {
                mat[i][j] = 1;
            });

        let mut best: (i64, Vec<i64>) = (i64::MAX, vec![]);
        let mut co: Vec<Vec<i64>> = vec![vec![]; n];

        for i in 0..n {
            co[i] = vec![i as i64];
        }

        for ph in 1..n {
            let mut w: Vec<i64> = mat[0].clone();
            let mut s: i64 = 0;
            let mut t: i64 = 0;
            for _ in 0..(n - ph) {
                w[t as usize] = i64::MIN;
                s = t;
                t = (0..w.len()).max_by_key(|i| w[*i]).unwrap() as i64;
                for i in 0..n {
                    w[i] += mat[t as usize][i];
                }
            }
            if w[t as usize] - mat[t as usize][t as usize] < best.0 {
                best = (
                    w[t as usize] - mat[t as usize][t as usize],
                    co[t as usize].clone(),
                );
            }
            let co_t = co[t as usize].clone();
            co[s as usize].extend(co_t.into_iter());
            for i in 0..n {
                mat[s as usize][i] += mat[t as usize][i];
            }
            for i in 0..n {
                mat[i][s as usize] = mat[s as usize][i];
            }
            mat[0][t as usize] = i64::MIN;
        }

        return best;
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 1;

    type ParsedInput = Vec<NamedConnection>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        connections: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let graph: IndexedGraph = connections.clone().into();

        let cut = graph.stoer_wagner_min_cut();
        let n = cut.1.len();
        Ok(n * (graph.connections.len() - n))
    }

    fn part_2(_: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        Ok(())
    }
}
