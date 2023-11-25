#![allow(unused_imports)]
use crate::utils::geometry::Vector;
use crate::utils::graph::DynamicGraph;
use crate::{Error, Puzzle};

use itertools::Itertools;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Resource {
    Ore,
    Clay,
    Obsidian,
    Geode,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct State {
    time_remaining: u8,
    resources: Vector<4, u8>,
    robots: Vector<4, u8>,
}

#[derive(Debug, Clone)]
pub struct Blueprint {
    id: u8,
    ore_costs: [u8; 4],
    clay_per_obsidian_robot: u8,
    obsidian_per_geode_robot: u8,
}

#[derive(Debug, Clone)]
struct BuildOrder {
    order: Vec<Resource>,
    state: State,
}

struct FullState<'a> {
    blueprint: &'a Blueprint,
    state: &'a State,
}

impl TryFrom<usize> for Resource {
    type Error = Error;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Resource::Ore),
            1 => Ok(Resource::Clay),
            2 => Ok(Resource::Obsidian),
            3 => Ok(Resource::Geode),
            _ => Err(Error::InvalidIndex(value)),
        }
    }
}
impl From<Resource> for usize {
    fn from(value: Resource) -> Self {
        match value {
            Resource::Ore => 0,
            Resource::Clay => 1,
            Resource::Obsidian => 2,
            Resource::Geode => 3,
        }
    }
}

impl Resource {
    fn unit_vec(&self) -> Vector<4, u8> {
        match self {
            Resource::Ore => [1, 0, 0, 0].into(),
            Resource::Clay => [0, 1, 0, 0].into(),
            Resource::Obsidian => [0, 0, 1, 0].into(),
            Resource::Geode => [0, 0, 0, 1].into(),
        }
    }
}

impl Blueprint {
    fn costs(&self, resource: Resource) -> Vector<4, u8> {
        use Resource::*;
        let index: usize = resource.into();
        [
            self.ore_costs[index],
            if matches!(resource, Obsidian) {
                self.clay_per_obsidian_robot
            } else {
                0
            },
            if matches!(resource, Geode) {
                self.obsidian_per_geode_robot
            } else {
                0
            },
            0,
        ]
        .into()
    }

    fn max_consumption(&self, resource: Resource) -> Option<u8> {
        match resource {
            Resource::Ore => {
                Some(self.ore_costs.iter().copied().max().unwrap())
            }
            Resource::Clay => Some(self.clay_per_obsidian_robot),
            Resource::Obsidian => Some(self.obsidian_per_geode_robot),
            Resource::Geode => None,
        }
    }

    fn build_orders(
        &self,
        time_remaining: u8,
    ) -> impl Iterator<Item = BuildOrder> + '_ {
        self.dijkstra_search(State::initial_state(time_remaining))
            .scan(Vec::new(), |prev_nodes, search_node| {
                let order: Vec<Resource> =
                    std::iter::successors(Some(&search_node), |(_, info)| {
                        info.backref
                            .as_ref()
                            .map(|edge| &prev_nodes[edge.initial_node])
                    })
                    .map(|(state, _)| state)
                    .tuple_windows()
                    .filter_map(|(after, before): (&State, &State)| {
                        (after.robots - before.robots)
                            .iter()
                            .enumerate()
                            .filter(|&(_i, &diff)| diff == 1)
                            .map(|(i, _diff)| {
                                i.try_into().expect(
                                    "Error backtracking the build order",
                                )
                            })
                            .exactly_one()
                            .ok()
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect();

                prev_nodes.push(search_node.clone());

                let (state, _info) = search_node;
                Some(BuildOrder { order, state })
            })
    }

    fn maximum_geodes(&self, time_remaining: u8, verbose: bool) -> u8 {
        self.build_orders(time_remaining)
            .filter(|b| b.state.time_remaining == 0)
            .max_by_key(|b| b.state.resources[Resource::Geode.into()])
            .into_iter()
            .inspect(|b| {
                if verbose {
                    println!(
                        "Best = {:?}, build order = {:?}",
                        b.state, b.order
                    )
                }
            })
            .map(|b| b.state.resources[Resource::Geode.into()])
            .exactly_one()
            .unwrap()
    }
}

impl State {
    fn initial_state(time_remaining: u8) -> Self {
        State {
            time_remaining,
            resources: [0; 4].into(),
            robots: [1, 0, 0, 0].into(),
        }
    }

    fn idle_until_end(&self) -> State {
        State {
            time_remaining: 0,
            resources: self.resources + self.time_remaining * self.robots,
            robots: self.robots,
        }
    }
}

impl<'a> FullState<'a> {
    fn time_until_robot(&self, resource: Resource) -> Option<u8> {
        self.state
            .resources
            .iter()
            .zip(self.state.robots.iter())
            .zip(self.blueprint.costs(resource).iter())
            .map(|((&have, &rate), &cost)| -> Option<u8> {
                let have: u8 = have;
                let rate: u8 = rate;
                let cost: u8 = cost;
                if have >= cost {
                    Some(1)
                } else if rate == 0 {
                    None
                } else {
                    let additional = cost.checked_sub(have).unwrap();
                    // TODO: Replace with additional.div_ceil once stable
                    // https://github.com/rust-lang/rust/issues/88581
                    let time_until_manufacturable =
                        (additional + rate - 1).checked_div(rate).unwrap();
                    let time_until_completed = time_until_manufacturable + 1;
                    Some(time_until_completed)
                }
            })
            .reduce(|a, b| match (a, b) {
                (Some(a), Some(b)) => Some(a.max(b)),
                _ => None,
            })
            .flatten()
    }

    fn after_new_robot(&self, resource: Resource) -> Option<State> {
        let time_required = self.time_until_robot(resource)?;
        let costs = self.blueprint.costs(resource);
        let time_remaining =
            self.state.time_remaining.checked_sub(time_required)?;
        let resources =
            self.state.resources + time_required * self.state.robots - costs;
        let robots = self.state.robots + resource.unit_vec();

        let new_state = State {
            time_remaining,
            resources,
            robots,
        };

        Some(new_state)
    }

    fn can_exclude(&self, resource: Resource) -> bool {
        let current_stockpile: u8 = self.state.resources[resource.into()];
        let current_rate: u8 = self.state.robots[resource.into()];
        if let Some(max_usage) = self.blueprint.max_consumption(resource) {
            if current_rate >= max_usage {
                return true;
            }

            let net_usage = max_usage - current_rate;
            if self.state.time_remaining.saturating_mul(net_usage)
                <= current_stockpile
            {
                return true;
            }
        }

        false
    }
}

impl DynamicGraph<State> for Blueprint {
    fn connections_from(&self, old_state: &State) -> Vec<(State, u64)> {
        let full_state = FullState {
            blueprint: self,
            state: old_state,
        };
        let build_robot = vec![
            Resource::Ore,
            Resource::Clay,
            Resource::Obsidian,
            Resource::Geode,
        ]
        .into_iter()
        .filter(|resource| !full_state.can_exclude(*resource))
        .filter_map(|resource| full_state.after_new_robot(resource))
        .map(|state| (state, false));

        let idle_until_end = (old_state.time_remaining > 0)
            .then(|| old_state.idle_until_end())
            .into_iter()
            .map(|state| (state, true));

        std::iter::empty()
            .chain(build_robot)
            .chain(idle_until_end)
            .enumerate()
            .filter_map(|(i, (state, must_be_first))| {
                (i == 0 || !must_be_first).then_some(state)
            })
            .inspect(|new_state| {
                assert_ne!(old_state.time_remaining, new_state.time_remaining)
            })
            .map(|new_state| {
                let time_spent =
                    old_state.time_remaining - new_state.time_remaining;
                let missing_max = 24 - new_state.robots[Resource::Geode.into()];
                (new_state, (time_spent as u64) * (missing_max as u64))
            })
            .collect()
    }

    fn heuristic_between(
        &self,
        _node_from: &State,
        _node_to: &State,
    ) -> Option<u64> {
        Some(1)
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<Blueprint>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines
            .flat_map(|line| line.split_ascii_whitespace())
            .chunks(32)
            .into_iter()
            .map(|chunk| -> Result<_, Error> {
                let chunk: Vec<&str> = chunk.collect();
                let id = chunk[1]
                    .strip_suffix(':')
                    .ok_or(Error::InvalidString(chunk[1].to_string()))?
                    .parse()?;
                let ore_costs = vec![6, 12, 18, 27]
                    .into_iter()
                    .map(|i| chunk[i].parse())
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .unwrap();
                let clay_per_obsidian_robot = chunk[21].parse()?;
                let obsidian_per_geode_robot = chunk[30].parse()?;
                Ok(Blueprint {
                    id,
                    ore_costs,
                    clay_per_obsidian_robot,
                    obsidian_per_geode_robot,
                })
            })
            .collect()
    }

    type Part1Result = u64;
    fn part_1(
        blueprints: &Self::ParsedInput,
    ) -> Result<Self::Part1Result, Error> {
        Ok(blueprints
            .iter()
            .map(|blueprint| {
                let max_geodes = blueprint.maximum_geodes(24, false);
                let quality_level = blueprint.id * max_geodes;
                quality_level as u64
            })
            .sum())
    }

    type Part2Result = u64;
    fn part_2(
        blueprints: &Self::ParsedInput,
    ) -> Result<Self::Part2Result, Error> {
        Ok(blueprints
            .iter()
            .take(3)
            .map(|blueprint| blueprint.maximum_geodes(32, false) as u64)
            .product())
    }
}
