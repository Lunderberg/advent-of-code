#![allow(unused_imports)]
use crate::utils::graph::DynamicGraph;
use crate::utils::Adjacency;
use crate::{Error, Puzzle};

use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use itertools::Itertools;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

#[derive(Debug, Clone)]
pub struct AmphipodDiagram {
    tiles: HashSet<Pos>,
    amphipods: HashMap<Pos, Amphipod>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Pos {
    i: i64,
    j: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Amphipod {
    A,
    B,
    C,
    D,
}

#[derive(Debug, Clone, Eq)]
struct AmphipodLayout {
    room_depth: usize,
    amphipods: HashMap<GraphNode, Amphipod>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum GraphNode {
    Hallway { i: i64 },
    Room { i: i64, steps_in: usize },
}

impl PartialEq for AmphipodLayout {
    fn eq(&self, other: &Self) -> bool {
        self.room_depth == other.room_depth && self.amphipods == other.amphipods
    }
}

impl Hash for AmphipodLayout {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.amphipods
            .iter()
            .sorted()
            .for_each(|pair| pair.hash(hasher))
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "({}, {})", self.i, self.j)
    }
}

impl From<Amphipod> for char {
    fn from(amphipod: Amphipod) -> char {
        match amphipod {
            Amphipod::A => 'A',
            Amphipod::B => 'B',
            Amphipod::C => 'C',
            Amphipod::D => 'D',
        }
    }
}

impl From<&Amphipod> for char {
    fn from(amphipod: &Amphipod) -> char {
        (*amphipod).into()
    }
}

impl Display for Amphipod {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let c: char = self.into();
        write!(f, "{c}")
    }
}

impl Display for AmphipodDiagram {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let left_char = self.tiles.iter().map(|pos| pos.i - 1).min().unwrap();

        let as_text = self
            .tiles
            .iter()
            .copied()
            .map(|pos| {
                (
                    pos,
                    self.amphipods
                        .get(&pos)
                        .map(|amph| amph.into())
                        .unwrap_or('.'),
                )
            })
            .chain(self.walls().map(|pos| (pos, '#')))
            .sorted_by_key(|(pos, _c)| pos.i)
            .into_grouping_map_by(|(pos, _c)| pos.j)
            .fold(Vec::new(), |mut acc, _line_num, (pos, c)| {
                while left_char + (acc.len() as i64) < pos.i {
                    acc.push(' ');
                }
                acc.push(c);
                acc
            })
            .into_iter()
            .sorted_by_key(|(line_num, _char_vec)| *line_num)
            .map(|(_line_num, char_vec)| -> String {
                char_vec.into_iter().collect()
            })
            .join("\n");

        write!(f, "{as_text}")
    }
}

impl Amphipod {
    fn room_num(&self) -> usize {
        use Amphipod::*;
        match self {
            A => 0,
            B => 1,
            C => 2,
            D => 3,
        }
    }

    fn step_cost(&self) -> u64 {
        use Amphipod::*;
        match self {
            A => 1,
            B => 10,
            C => 100,
            D => 1000,
        }
    }
}

impl AmphipodDiagram {
    fn walls(&self) -> impl Iterator<Item = Pos> + '_ {
        self.tiles
            .iter()
            .flat_map(|pos| {
                Adjacency::Queen
                    .adjacent(pos.i, pos.j)
                    .map(|(i, j)| Pos { i, j })
            })
            .filter(move |pos| !self.tiles.contains(pos))
            .unique()
    }

    fn room_depth(&self) -> usize {
        self.tiles
            .iter()
            .map(|pos| pos.j)
            .minmax()
            .into_option()
            .map(|(a, b)| (b - a) as usize)
            .unwrap()
    }

    fn extend_part_2(&self) -> Self {
        let inserted_lines = "  #D#C#B#A#\n  #D#B#A#C#".lines();

        let orig = format!("{self}");
        let mut lines = orig.lines();

        lines
            .by_ref()
            .take(3)
            .chain(inserted_lines)
            .collect::<Vec<_>>()
            .into_iter()
            .chain(lines)
            .collect()
    }
}

impl<'a> FromIterator<&'a str> for AmphipodDiagram {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let active_spaces: Vec<_> = iter
            .into_iter()
            .enumerate()
            .flat_map(|(j, line)| {
                line.chars().enumerate().map(move |(i, c)| {
                    (
                        Pos {
                            i: i as i64,
                            j: j as i64,
                        },
                        c,
                    )
                })
            })
            .filter_map(|(pos, c)| match c {
                '.' => Some((pos, None)),
                'A' => Some((pos, Some(Amphipod::A))),
                'B' => Some((pos, Some(Amphipod::B))),
                'C' => Some((pos, Some(Amphipod::C))),
                'D' => Some((pos, Some(Amphipod::D))),
                _ => None,
            })
            .collect();

        let tiles = active_spaces.iter().map(|(pos, _)| pos).copied().collect();
        let amphipods = active_spaces
            .into_iter()
            .filter_map(|(pos, opt_amphipod)| opt_amphipod.map(|a| (pos, a)))
            .collect();

        Self { tiles, amphipods }
    }
}

impl AmphipodLayout {
    const HALLWAY_MIN: i64 = 1;
    const HALLWAY_MAX: i64 = 11;
    const ROOM_LOCS: [i64; 4] = [3, 5, 7, 9];

    fn all_nodes_by_depth(
        room_depth: usize,
    ) -> impl Iterator<Item = GraphNode> {
        use GraphNode::*;

        let hallway =
            (Self::HALLWAY_MIN..=Self::HALLWAY_MAX).map(|i| Hallway { i });
        let rooms = Self::ROOM_LOCS
            .iter()
            .copied()
            .cartesian_product(0..room_depth)
            .map(|(i, steps_in)| Room { i, steps_in });

        hallway.chain(rooms)
    }

    fn all_nodes(&self) -> impl Iterator<Item = GraphNode> {
        Self::all_nodes_by_depth(self.room_depth)
    }

    fn node_pos(node: &GraphNode) -> Pos {
        use GraphNode::*;
        match node {
            Hallway { i } => Pos { i: *i, j: 1 },
            Room { i, steps_in } => Pos {
                i: *i,
                j: 2 + (*steps_in as i64),
            },
        }
    }

    fn allowed_moves(
        &self,
    ) -> impl Iterator<Item = (GraphNode, GraphNode)> + '_ {
        use GraphNode::*;

        self.amphipods
            .iter()
            .flat_map(move |(current, amph)| {
                self.all_nodes().map(move |target| (*current, target, amph))
            })
            // Will not move to its own location
            .filter(|(current, target, _amph)| current != target)
            // Will never move into a non-target room
            .filter(|(_current, target, amph)| match target {
                Room { i, .. } => Self::ROOM_LOCS[amph.room_num()] == *i,
                _ => true,
            })
            // Will never stop in front of a room
            .filter(|(_current, target, _amph)| match target {
                Hallway { i, .. } => {
                    Self::ROOM_LOCS.iter().all(|room_pos| i != room_pos)
                }
                _ => true,
            })
            // After in the hallway, will only move into a room.
            .filter(|(current, target, _amph)| match (current, target) {
                (Hallway { .. }, Room { .. }) => true,
                (Hallway { .. }, _) => false,
                _ => true,
            })
            // Will not move into a room that is occupied by other types.
            .filter(move |(_current, target, amph)| match target {
                Room { i, .. } => (0..self.room_depth)
                    .map(|steps_in| Room { i: *i, steps_in })
                    .all(move |node| {
                        self.amphipods
                            .get(&node)
                            .map(|other_amph| **amph == *other_amph)
                            .unwrap_or(true)
                    }),
                _ => true,
            })
            // Will always move to the last unoccupied space in a room
            .filter(move |(_current, target, _amph)| match target {
                Room { i, steps_in } => ((steps_in + 1)..self.room_depth)
                    .map(|deeper| Room {
                        i: *i,
                        steps_in: deeper,
                    })
                    .all(|node| self.amphipods.contains_key(&node)),
                _ => true,
            })
            // No other amphipods along the path to the target location
            .filter(move |(current, target, _amph)| {
                Self::steps_to(current, target)
                    .all(move |node| self.amphipods.get(&node).is_none())
            })
            .map(|(current, target, _amph)| (current, target))
    }

    // Moves an amphipod from the current location to the target
    // location.  If the current location is unoccupied, or if the
    // target location is occupied, will make no changes and return an
    // error.  Does not check that the path is clear.
    fn apply_move(
        &mut self,
        current: &GraphNode,
        target: &GraphNode,
    ) -> Result<(), Error> {
        if self.amphipods.contains_key(target) {
            return Err(Error::AmphipodAtTargetLocation);
        }

        self.amphipods
            .remove_entry(current)
            .ok_or(Error::NoAmphipodAtCurrentLocation)
            .map(|(_pos, amph)| {
                self.amphipods.insert(*target, amph);
            })
    }

    fn after_move(
        &self,
        current: &GraphNode,
        target: &GraphNode,
    ) -> Result<Self, Error> {
        let mut out: Self = self.clone();
        out.apply_move(current, target)?;
        Ok(out)
    }

    // Includes the target point, but not the current point.
    fn steps_to(
        current: &GraphNode,
        target: &GraphNode,
    ) -> impl Iterator<Item = GraphNode> {
        use GraphNode::*;

        fn iterate_towards<T>(a: T, b: T) -> impl Iterator<Item = T>
        where
            T: num::Integer + Copy,
        {
            let successor = if a < b {
                |x| x + T::one()
            } else {
                |x| x - T::one()
            };
            std::iter::successors(Some(a), move |&prev| {
                (prev != b).then(move || successor(prev))
            })
        }

        fn steps_to_entranceway(
            i: i64,
            steps_in: usize,
        ) -> impl Iterator<Item = GraphNode> {
            (0..=steps_in).rev().map(move |s| Room { i, steps_in: s })
        }

        fn steps_in_hallway(
            i_start: i64,
            i_end: i64,
        ) -> impl Iterator<Item = GraphNode> {
            iterate_towards(i_start, i_end).map(|i| Hallway { i })
        }

        fn steps_from_entranceway(
            i: i64,
            steps_in: usize,
        ) -> impl Iterator<Item = GraphNode> {
            (0..=steps_in).map(move |s| Room { i, steps_in: s })
        }

        fn steps_within_room(
            i: i64,
            initial_steps_in: usize,
            final_steps_in: usize,
        ) -> impl Iterator<Item = GraphNode> {
            iterate_towards(initial_steps_in, final_steps_in)
                .map(move |steps_in| Room { i, steps_in })
        }

        let steps: Box<dyn Iterator<Item = GraphNode>> = match (current, target)
        {
            (
                Room {
                    i: i_room,
                    steps_in,
                },
                Hallway { i: i_hall },
            ) => Box::new(
                steps_to_entranceway(*i_room, *steps_in)
                    .chain(steps_in_hallway(*i_room, *i_hall)),
            ),

            (
                Hallway { i: i_hall },
                Room {
                    i: i_room,
                    steps_in,
                },
            ) => Box::new(
                steps_in_hallway(*i_hall, *i_room)
                    .chain(steps_from_entranceway(*i_room, *steps_in)),
            ),

            (Hallway { i: i_start }, Hallway { i: i_end }) => {
                Box::new(steps_in_hallway(*i_start, *i_end))
            }

            (
                Room {
                    i: i_start,
                    steps_in: s_start,
                },
                Room {
                    i: i_end,
                    steps_in: s_end,
                },
            ) => {
                if i_start == i_end {
                    Box::new(steps_within_room(*i_start, *s_start, *s_end))
                } else {
                    Box::new(
                        steps_to_entranceway(*i_start, *s_start)
                            .chain(steps_in_hallway(*i_start, *i_end))
                            .chain(steps_from_entranceway(*i_end, *s_end)),
                    )
                }
            }
        };

        steps.skip(1)
    }

    // Includes target node, but not the current node.
    fn num_steps_to(current: &GraphNode, target: &GraphNode) -> u64 {
        use GraphNode::*;

        fn abs_diff<T>(a: T, b: T) -> T
        where
            T: num::Integer,
        {
            if a < b {
                b - a
            } else {
                a - b
            }
        }

        match (current, target) {
            (
                Room {
                    i: i_room,
                    steps_in,
                },
                Hallway { i: i_hall },
            ) => ((steps_in + 1) as u64) + (abs_diff(*i_room, *i_hall) as u64),

            (
                Hallway { i: i_hall },
                Room {
                    i: i_room,
                    steps_in,
                },
            ) => ((steps_in + 1) as u64) + (abs_diff(*i_room, *i_hall) as u64),

            (Hallway { i: i_start }, Hallway { i: i_end }) => {
                abs_diff(*i_start, *i_end) as u64
            }

            (
                Room {
                    i: i_start,
                    steps_in: s_start,
                },
                Room {
                    i: i_end,
                    steps_in: s_end,
                },
            ) => {
                if i_start == i_end {
                    abs_diff(*s_start, *s_end) as u64
                } else {
                    ((s_start + 1) as u64)
                        + (abs_diff(*i_start, *i_end) as u64)
                        + ((s_end + 1) as u64)
                }
            }
        }
    }

    fn cost_to_apply(
        &self,
        current: &GraphNode,
        target: &GraphNode,
    ) -> Result<u64, Error> {
        self.amphipods
            .get(current)
            .ok_or(Error::NoAmphipodAtCurrentLocation)
            .map(|amph| amph.step_cost() * Self::num_steps_to(current, target))
    }

    fn target_arrangement(&self) -> Result<Self, Error> {
        let (in_position, out_of_position): (HashMap<_, _>, HashMap<_, _>) =
            self.amphipods.iter().partition(|(pos, amph)| match pos {
                GraphNode::Room { i, .. } => {
                    Self::ROOM_LOCS[amph.room_num()] == *i
                }
                _ => false,
            });

        let fold_func =
            |res_acc: Result<HashMap<GraphNode, Amphipod>, Error>,
             (_pos, amph): (GraphNode, Amphipod)| {
                res_acc.and_then(|mut acc| {
                    let i = AmphipodLayout::ROOM_LOCS[amph.room_num()];
                    (0..self.room_depth)
                        .map(|steps_in| GraphNode::Room { i, steps_in })
                        .find(|pos| !acc.contains_key(pos))
                        .ok_or(Error::TooManyAmphipodsForRoom)
                        .map(move |pos| {
                            acc.insert(pos, amph);
                            acc
                        })
                })
            };

        let amphipods: HashMap<GraphNode, Amphipod> = out_of_position
            .into_iter()
            .fold(Ok(in_position), fold_func)?;

        Ok(Self {
            amphipods,
            room_depth: self.room_depth,
        })
    }
}

impl DynamicGraph<Self> for AmphipodLayout {
    fn connections_from(&self, node: &Self) -> Vec<(Self, u64)> {
        node.allowed_moves()
            .map(|(from, to)| {
                let amph = node.amphipods[&from];
                let cost = amph.step_cost()
                    * (Self::steps_to(&from, &to).count() as u64);
                node.after_move(&from, &to).map(|graph| (graph, cost))
            })
            .collect::<Result<_, _>>()
            .unwrap()
    }

    fn heuristic_between(&self, a: &Self, b: &Self) -> Option<u64> {
        let possible_target_map: HashMap<Amphipod, HashSet<GraphNode>> = b
            .amphipods
            .iter()
            .map(|(&pos, &amph)| (amph, pos))
            .into_grouping_map()
            .collect();

        Some(
            a.amphipods
                .iter()
                .map(|(a_pos, amph)| {
                    possible_target_map[amph]
                        .iter()
                        .map(|b_pos| a.cost_to_apply(a_pos, b_pos))
                        .fold_ok(u64::MAX, |acc, cost| acc.min(cost))
                })
                .fold_ok(0, |acc, cost| acc + cost)
                .unwrap(),
        )
    }
}

impl From<&AmphipodDiagram> for AmphipodLayout {
    fn from(diagram: &AmphipodDiagram) -> AmphipodLayout {
        let room_depth = diagram.room_depth();

        let amphipods = Self::all_nodes_by_depth(room_depth)
            .flat_map(|node| {
                diagram
                    .amphipods
                    .get(&Self::node_pos(&node))
                    .copied()
                    .map(|amph| (node, amph))
            })
            .collect();
        Self {
            amphipods,
            room_depth,
        }
    }
}

impl From<AmphipodDiagram> for AmphipodLayout {
    fn from(diagram: AmphipodDiagram) -> AmphipodLayout {
        (&diagram).into()
    }
}

impl From<&AmphipodLayout> for AmphipodDiagram {
    fn from(graph: &AmphipodLayout) -> AmphipodDiagram {
        let amphipods = graph
            .amphipods
            .iter()
            .map(|(node, amph)| (AmphipodLayout::node_pos(node), *amph))
            .collect();

        let tiles = graph
            .all_nodes()
            .map(|node| AmphipodLayout::node_pos(&node))
            .collect();

        Self { tiles, amphipods }
    }
}

impl From<AmphipodLayout> for AmphipodDiagram {
    fn from(graph: AmphipodLayout) -> AmphipodDiagram {
        (&graph).into()
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = AmphipodDiagram;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines.collect())
    }

    type Part1Result = u64;
    fn part_1(parsed: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        let initial: AmphipodLayout = parsed.into();
        let target = initial.target_arrangement()?;

        let path = initial.shortest_path(initial.clone(), target)?;

        Ok(path.into_iter().map(|(_state, cost)| cost).sum::<u64>())
    }

    type Part2Result = u64;
    fn part_2(parsed: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        let diagram = parsed.extend_part_2();

        let initial: AmphipodLayout = diagram.into();
        let target = initial.target_arrangement()?;

        let path = initial.shortest_path(initial.clone(), target)?;

        Ok(path.into_iter().map(|(_state, cost)| cost).sum::<u64>())
    }
}
