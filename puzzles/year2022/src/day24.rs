use aoc_utils::prelude::*;

use crate::utils::Adjacency;

use console::Style;
use std::fmt::{Display, Formatter};

type Point = Vector<2, i64>;

pub struct StormSystem {
    map: GridMap<Tile>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum Tile {
    Wall,
    Ground,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct State {
    pos: Point,
    time: i64,
    phase: u8,
}

#[derive(Clone)]
enum DisplayTile {
    Tile(Tile),
    Overlapping(u8),
    Elf,
    Goal,
}

impl Tile {
    fn velocity(&self) -> Option<Vector<2, i64>> {
        match self {
            Tile::Left => Some([-1, 0].into()),
            Tile::Right => Some([1, 0].into()),
            Tile::Up => Some([0, -1].into()),
            Tile::Down => Some([0, 1].into()),
            _ => None,
        }
    }
}

impl StormSystem {
    fn initial(&self) -> Point {
        [1, 0].into()
    }

    fn goal(&self) -> Point {
        let x_size = self.map.x_size as i64;
        let y_size = self.map.y_size as i64;

        [x_size - 2, y_size - 1].into()
    }

    fn mod_storm(&self, pos: Point) -> Point {
        let (width, height) = self.map.shape();
        let inner_width = (width - 2) as i64;
        let inner_height = (height - 2) as i64;
        [
            (pos.x() - 1).rem_euclid(inner_width) + 1,
            (pos.y() - 1).rem_euclid(inner_height) + 1,
        ]
        .into()
    }

    fn has_storm(&self, pos: Point, time: i64) -> bool {
        let (width, height) = self.map.shape();
        let width = width as i64;
        let height = height as i64;

        (1..(width - 1)).contains(&pos.x())
            && (1..(height - 1)).contains(&pos.y())
            && vec![Tile::Left, Tile::Right, Tile::Down, Tile::Up]
                .into_iter()
                .any(|tile| -> bool {
                    let velocity = tile.velocity().unwrap();
                    let orig_pos = pos - time * velocity;
                    let rel_pos: Point = [
                        (orig_pos.x() - 1).rem_euclid(width - 2) + 1,
                        (orig_pos.y() - 1).rem_euclid(height - 2) + 1,
                    ]
                    .into();
                    self.map[rel_pos] == tile
                })
    }

    fn at_time(&self, state: &State) -> GridMap<DisplayTile> {
        let goal = self.goal();
        let special = if goal == state.pos {
            vec![(state.pos, DisplayTile::Elf)]
        } else {
            vec![(state.pos, DisplayTile::Elf), (goal, DisplayTile::Goal)]
        };

        self.map
            .iter_vec()
            .filter(|(_, tile)| !matches!(tile, Tile::Ground))
            .map(|(pos, tile)| {
                (
                    tile.velocity()
                        .map(|velocity| {
                            self.mod_storm(pos + state.time * velocity)
                        })
                        .unwrap_or(pos),
                    tile.clone(),
                )
            })
            .into_group_map()
            .into_iter()
            .map(|(pos, vec_tile)| -> (Vector<2, i64>, DisplayTile) {
                if vec_tile.len() == 1 {
                    let tile = vec_tile.into_iter().next().unwrap();
                    (pos, DisplayTile::Tile(tile))
                } else {
                    (pos, DisplayTile::Overlapping(vec_tile.len() as u8))
                }
            })
            .chain(special)
            .collect_resized_grid_map(DisplayTile::Tile(Tile::Ground))
    }

    fn search(&self, initial_phase: u8) -> Result<u64, Error> {
        let initial = self.initial();
        let goal = self.goal();
        let min_traversal = initial.manhattan_dist(&goal) as u64;

        let heuristic = |state: &State| -> Option<u64> {
            match state.phase {
                0 => Some(state.pos.manhattan_dist(&goal) as u64),
                1 => Some(
                    state.pos.manhattan_dist(&initial) as u64 + min_traversal,
                ),
                2 => Some(
                    state.pos.manhattan_dist(&goal) as u64 + 2 * min_traversal,
                ),
                _ => panic!("Unknown phase {}", state.phase),
            }
        };

        let (node, info, path) = self
            .a_star_search(
                State {
                    pos: self.initial(),
                    time: 0,
                    phase: initial_phase,
                },
                heuristic,
            )
            .scan(Vec::new(), |nodes, node_info| {
                nodes.push(node_info.clone());
                let path: Vec<_> = std::iter::successors(
                    Some(node_info.clone()),
                    |(_, info)| {
                        info.backref
                            .as_ref()
                            .map(|edge| nodes[edge.initial_node].clone())
                    },
                )
                .map(|(state, _)| state.pos)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();

                let (node, info) = node_info;
                Some((node, info, path))
            })
            .find(|(node, _, _)| node.pos == goal && node.phase == 0)
            .ok_or(Error::NoPathToDest)?;

        println!(
            "Reached {node:?} after {} minutes, \
             minimum {} minutes remaining\n{}\n\n",
            info.initial_to_node,
            info.heuristic,
            self.at_time(&node).map(|(pos, tile)| {
                let style = if path.contains(&pos) {
                    Style::new().green().bright().bold()
                } else {
                    Style::new().red()
                };
                style.apply_to(tile)
            }),
        );

        Ok(info.initial_to_node)
    }
}

impl DynamicGraph<State> for StormSystem {
    fn connections_from(&self, node: &State) -> Vec<(State, u64)> {
        if node.phase == 2 && node.pos == self.goal() {
            vec![(
                State {
                    pos: node.pos,
                    phase: 1,
                    time: node.time,
                },
                0,
            )]
        } else if node.phase == 1 && node.pos == self.initial() {
            vec![(
                State {
                    pos: node.pos,
                    phase: 0,
                    time: node.time,
                },
                0,
            )]
        } else {
            let time = node.time + 1;
            let phase = node.phase;
            self.map
                .adjacent_points(node.pos, Adjacency::Rook)
                .filter(|pos| self.map[*pos] != Tile::Wall)
                .map(|pos| -> Point { pos.as_vec(&self.map) })
                .chain(std::iter::once(node.pos))
                .filter(|pos| !self.has_storm(*pos, time))
                .map(|pos| (State { pos, time, phase }, 1))
                .collect()
        }
    }

    fn heuristic_between(
        &self,
        node_from: &State,
        node_to: &State,
    ) -> Option<u64> {
        Some(node_from.pos.manhattan_dist(&node_to.pos) as u64)
    }
}

impl std::str::FromStr for Tile {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = s.chars().exactly_one_or_err()?;
        match c {
            '#' => Ok(Tile::Wall),
            '.' => Ok(Tile::Ground),
            '>' => Ok(Tile::Right),
            '<' => Ok(Tile::Left),
            '^' => Ok(Tile::Up),
            'v' => Ok(Tile::Down),
            _ => Err(Error::UnknownChar(c)),
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Tile::Wall => '#',
            Tile::Ground => '.',
            Tile::Right => '>',
            Tile::Left => '<',
            Tile::Up => '^',
            Tile::Down => 'v',
        };
        write!(f, "{c}")
    }
}

impl Display for StormSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.map)
    }
}

impl Display for DisplayTile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayTile::Tile(tile) => write!(f, "{tile}"),
            DisplayTile::Elf => write!(f, "E"),
            DisplayTile::Goal => write!(f, "G"),
            DisplayTile::Overlapping(num) if *num < 10 => write!(f, "{num}"),
            DisplayTile::Overlapping(_) => {
                panic!("At most 4 storms should overlap")
            }
        }
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 6;

    type ParsedInput = StormSystem;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let map = lines.collect();
        Ok(StormSystem { map })
    }

    fn part_1(
        storms: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        storms.search(0)
    }

    fn part_2(
        storms: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        storms.search(2)
    }
}
