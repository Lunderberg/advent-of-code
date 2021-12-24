#![allow(unused_imports)]
use crate::utils::Adjacency;
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};

use itertools::Itertools;

pub struct Day23;

#[derive(Debug, Clone)]
struct Diagram {
    tiles: HashSet<Pos>,
    amphipods: HashMap<Pos, Amphipod>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Pos {
    i: i64,
    j: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Amphipod {
    A,
    B,
    C,
    D,
}

#[derive(Debug)]
struct Graph {
    amphipods: HashMap<GraphNode, Amphipod>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GraphNode {
    Hallway { i: i64 },
    Room { i: i64, steps_in: usize },
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
        write!(f, "{}", c)
    }
}

impl Display for Diagram {
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

        write!(f, "{}", as_text)
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

// impl Pos {
//     fn adjacent(&self) -> impl Iterator<Item = Self> {
//         Adjacency::Rook
//             .adjacent(self.i, self.j)
//             .map(|(i, j)| Pos { i, j })
//     }
// }

impl Diagram {
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
}

impl Graph {
    const HALLWAY_MIN: i64 = 1;
    const HALLWAY_MAX: i64 = 11;
    const ROOM_LOCS: [i64; 4] = [3, 5, 7, 9];
    const ROOM_DEPTH: usize = 2;

    fn all_nodes() -> impl Iterator<Item = GraphNode> {
        use GraphNode::*;

        let hallway =
            (Self::HALLWAY_MIN..=Self::HALLWAY_MAX).map(|i| Hallway { i });
        let rooms = Self::ROOM_LOCS
            .iter()
            .copied()
            .cartesian_product(0..Self::ROOM_DEPTH)
            .map(|(i, steps_in)| Room { i, steps_in });

        hallway.chain(rooms)
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
            .flat_map(|(current, amph)| {
                Self::all_nodes().map(move |target| (*current, target, amph))
            })
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
                Room { i, .. } => (0..Self::ROOM_DEPTH)
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
                Room { i, steps_in } => ((steps_in + 1)..Self::ROOM_DEPTH)
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

    fn cost_heuristic(&self) -> u64 {
        self.amphipods
            .iter()
            .map(|(node, amph)| {
                use GraphNode::*;

                let target_room = Self::ROOM_LOCS[amph.room_num()];

                let target = match node {
                    Room { i, .. } => {
                        if *i == target_room {
                            *node
                        } else {
                            Room {
                                i: target_room,
                                steps_in: 0,
                            }
                        }
                    }
                    _ => Room {
                        i: target_room,
                        steps_in: 0,
                    },
                };

                Graph::num_steps_to(node, &target) * amph.step_cost()
            })
            .sum()
    }
}

impl From<&Diagram> for Graph {
    fn from(diagram: &Diagram) -> Graph {
        let amphipods = Self::all_nodes()
            .flat_map(|node| {
                diagram
                    .amphipods
                    .get(&Self::node_pos(&node))
                    .copied()
                    .map(|amph| (node, amph))
            })
            .collect();
        Self { amphipods }
    }
}

impl From<Diagram> for Graph {
    fn from(diagram: Diagram) -> Graph {
        (&diagram).into()
    }
}

impl From<&Graph> for Diagram {
    fn from(graph: &Graph) -> Diagram {
        let amphipods = graph
            .amphipods
            .iter()
            .map(|(node, amph)| (Graph::node_pos(node), *amph))
            .collect();

        let tiles = Graph::all_nodes()
            .map(|node| Graph::node_pos(&node))
            .collect();

        Self { tiles, amphipods }
    }
}

impl From<Graph> for Diagram {
    fn from(graph: Graph) -> Diagram {
        (&graph).into()
    }
}

impl Day23 {
    fn parse_diagram(&self) -> Result<Diagram, Error> {
        let puzzle_input = self.puzzle_input(PuzzleInput::Example(0))?;
        //let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let active_spaces: Vec<(Pos, Option<Amphipod>)> = puzzle_input
            .lines()
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

        Ok(Diagram { tiles, amphipods })
    }
}

impl Puzzle for Day23 {
    fn day(&self) -> i32 {
        23
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let diagram = self.parse_diagram()?;

        println!("Diagram:\n{}", diagram);

        let graph: Graph = diagram.into();

        graph
            .allowed_moves()
            .sorted_by_key(|(src, dest)| {
                let src = Graph::node_pos(src);
                let dest = Graph::node_pos(dest);
                [src.i, src.j, dest.i, dest.j]
            })
            .for_each(|(src, dest)| {
                println!("Move from {:?} to {:?}", src, dest)
            });

        println!(
            "Graph: {:?}",
            graph
                .amphipods
                .iter()
                .sorted_by_key(|(node, _amph)| {
                    let pos = Graph::node_pos(node);
                    (pos.i, pos.j)
                })
                .collect::<Vec<_>>()
        );

        println!("Heuristic: {}", graph.cost_heuristic());

        // let round_trip: Diagram = graph.into();

        // println!("Round trip:\n{}", round_trip);

        let result = ();
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let result = ();
        Ok(Box::new(result))
    }
}
