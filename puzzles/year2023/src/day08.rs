use num::integer::{gcd, lcm};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use aoc_utils::prelude::*;

#[derive(Debug, Clone, Copy)]
enum Step {
    Right,
    Left,
}

#[derive(Debug, Clone)]
struct Node<T> {
    name: String,
    left: T,
    right: T,
}

#[derive(Debug, Clone)]
pub struct Map {
    directions: Vec<Step>,
    nodes: Vec<Node<String>>,
}

#[derive(Debug)]
pub struct IndexedMap {
    directions: Vec<Step>,
    nodes: Vec<Node<usize>>,
}

#[derive(Debug)]
struct CycleAnalysis {
    cycle_start: usize,
    period: usize,
    goal_before_first_cycle: Vec<usize>,
    goal_each_cycle: Vec<usize>,
}

impl TryFrom<char> for Step {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'L' => Ok(Step::Left),
            'R' => Ok(Step::Right),
            _ => Err(Error::UnknownChar(value)),
        }
    }
}

impl FromStr for Node<String> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, _equal, left, right) = s
            .split_ascii_whitespace()
            .collect_tuple()
            .ok_or(Error::WrongIteratorSize)?;
        let left = left.trim_start_matches('(').trim_end_matches(',');
        let right = right.trim_end_matches(')');
        let name = name.to_string();
        let left = left.to_string();
        let right = right.to_string();
        Ok(Self { name, left, right })
    }
}

impl TryFrom<Map> for IndexedMap {
    type Error = Error;

    fn try_from(value: Map) -> Result<Self, Self::Error> {
        let name_lookup: HashMap<_, _> = value
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (node.name.clone(), i))
            .collect();

        let get_index = |name: String| -> Result<usize, Error> {
            name_lookup
                .get(&name)
                .cloned()
                .ok_or(Error::InvalidString(name))
        };

        let nodes = value
            .nodes
            .into_iter()
            .map(|node| -> Result<_, Error> {
                Ok(Node {
                    name: node.name,
                    left: get_index(node.left)?,
                    right: get_index(node.right)?,
                })
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            directions: value.directions,
            nodes,
        })
    }
}

impl IndexedMap {
    fn cycle_analysis(
        &self,
        start: usize,
        goal: &HashSet<usize>,
    ) -> CycleAnalysis {
        let mut cycle_start: Option<usize> = None;

        let states: Vec<_> = self
            .directions
            .iter()
            .enumerate()
            .cycle()
            .scan(start, |state, (i_cycle, direction)| {
                let before = *state;
                let node = &self.nodes[before];
                *state = match direction {
                    Step::Left => node.left,
                    Step::Right => node.right,
                };
                Some((before, i_cycle))
            })
            .enumerate()
            .scan(
                HashMap::<(usize, usize), usize>::new(),
                |seen, (i, state)| {
                    if let Some(prev) = seen.get(&state) {
                        cycle_start = Some(*prev);
                        None
                    } else {
                        seen.insert(state, i);
                        Some(state)
                    }
                },
            )
            .collect();

        let cycle_start =
            cycle_start.expect("Exited loop without setting cycle_start");
        let period = states.len() - cycle_start;
        println!(
            "Starting from {start}, \
             found repeated state at {}, \
             with same state as {cycle_start}, \
             resulting in period of {period}",
            states.len()
        );

        states
            .iter()
            .enumerate()
            .filter(|(_, (loc, _))| goal.contains(loc))
            .for_each(|(i, (loc, _))| {
                println!(
                    "From initial {start}, \
                     found target {loc} on step {i}"
                )
            });

        let goal_before_first_cycle = states
            .iter()
            .take(cycle_start)
            .enumerate()
            .filter(|(_, (loc, _i_cycle))| goal.contains(loc))
            .map(|(i, _)| i)
            .collect();

        let goal_each_cycle = states
            .iter()
            .skip(cycle_start)
            .enumerate()
            .filter(|(_, (loc, _i_cycle))| goal.contains(loc))
            .map(|(i, _)| i)
            .collect();

        CycleAnalysis {
            cycle_start,
            period,
            goal_before_first_cycle,
            goal_each_cycle,
        }
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 2;

    type ParsedInput = Map;
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let directions = lines
            .next()
            .unwrap()
            .chars()
            .map(|c| c.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        lines.next();
        let nodes = lines
            .map(|line| line.parse())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Map { directions, nodes })
    }

    fn part_1(map: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        let map: IndexedMap = map.clone().try_into()?;

        let start: usize = map
            .nodes
            .iter()
            .enumerate()
            .find(|(_, node)| node.name == "AAA")
            .map(|(i, _)| i)
            .ok_or(Error::InvalidString("AAA".to_string()))?;

        let goal: usize = map
            .nodes
            .iter()
            .enumerate()
            .find(|(_, node)| node.name == "ZZZ")
            .map(|(i, _)| i)
            .ok_or(Error::InvalidString("AAA".to_string()))?;

        let num_steps = map
            .directions
            .iter()
            .cycle()
            .scan(start, |state, direction| {
                let before = *state;
                let node = &map.nodes[before];
                *state = match direction {
                    Step::Left => node.left,
                    Step::Right => node.right,
                };
                Some(before)
            })
            .enumerate()
            .find(|(_, node_index)| *node_index == goal)
            .map(|(num_steps, _)| num_steps)
            .unwrap();

        Ok(num_steps)
    }

    fn part_2(map: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        let map: IndexedMap = map.clone().try_into()?;

        let start: Vec<usize> = map
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.name.ends_with('A'))
            .map(|(i, _)| i)
            .collect();

        let goal: HashSet<usize> = map
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.name.ends_with('Z'))
            .map(|(i, _)| i)
            .collect();

        let analysis: Vec<_> = start
            .iter()
            .map(|&start| map.cycle_analysis(start, &goal))
            .collect();

        let (offset, period) = analysis
            .iter()
            .map(|cycle| {
                assert_eq!(cycle.goal_before_first_cycle.len(), 0);
                assert_eq!(cycle.goal_each_cycle.len(), 1);

                let offset = cycle.cycle_start + cycle.goal_each_cycle[0];
                (offset, cycle.period)
            })
            .fold((0, 1), |(offset_a, period_a), (offset_b, period_b)| {
                let offset_a = offset_a % period_a;
                let offset_b = offset_b % period_b;

                let new_period = lcm(period_a, period_b);
                let new_offset = (offset_a * period_b + offset_b * period_a)
                    / gcd(period_a, period_b);

                (new_offset, new_period)
            });

        let offset = if analysis.iter().any(|cycle| offset < cycle.cycle_start)
        {
            offset + period
        } else {
            offset
        };

        Ok(offset)
    }
}
