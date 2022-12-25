#![allow(unused_imports)]
use crate::utils::geometry::Vector;
use crate::utils::{GridMap, GridPos};
use crate::{Error, Puzzle};

use itertools::{Either, Itertools};
use num::integer::gcd;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug)]
pub struct MonkeyMap {
    map: GridMap<Tile>,
    teleports: HashMap<State, State>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tile {
    Open,
    Solid,
    Wrap,
}

#[derive(Debug, Clone)]
pub enum Command {
    TurnLeft,
    TurnRight,
    Advance(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Facing {
    Right,
    Down,
    Left,
    Up,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State {
    loc: GridPos,
    facing: Facing,
}

impl Facing {
    fn value(&self) -> u64 {
        match self {
            Facing::Right => 0,
            Facing::Down => 1,
            Facing::Left => 2,
            Facing::Up => 3,
        }
    }

    fn reverse(&self) -> Facing {
        match self {
            Facing::Right => Facing::Left,
            Facing::Down => Facing::Up,
            Facing::Left => Facing::Right,
            Facing::Up => Facing::Down,
        }
    }

    fn iter() -> impl Iterator<Item = Self> {
        vec![Facing::Right, Facing::Down, Facing::Left, Facing::Up].into_iter()
    }

    fn direction(&self) -> Vector<2, i64> {
        match self {
            Facing::Right => [1, 0].into(),
            Facing::Down => [0, 1].into(),
            Facing::Left => [-1, 0].into(),
            Facing::Up => [0, -1].into(),
        }
    }

    fn after_turn_left(&self) -> Self {
        match self {
            Facing::Right => Facing::Up,
            Facing::Down => Facing::Right,
            Facing::Left => Facing::Down,
            Facing::Up => Facing::Left,
        }
    }

    fn after_turn_right(&self) -> Self {
        match self {
            Facing::Right => Facing::Down,
            Facing::Down => Facing::Left,
            Facing::Left => Facing::Up,
            Facing::Up => Facing::Right,
        }
    }
}

impl State {
    fn apply(&self, map: &MonkeyMap, command: Command) -> Self {
        match command {
            Command::TurnLeft => State {
                loc: self.loc,
                facing: self.facing.after_turn_left(),
            },
            Command::TurnRight => State {
                loc: self.loc,
                facing: self.facing.after_turn_right(),
            },
            Command::Advance(dist) => std::iter::successors(
                Some(self.clone()),
                |prev| -> Option<State> {
                    map.teleports.get(prev).cloned().or_else(|| {
                        let vec =
                            prev.loc.as_vec(&map.map) + prev.facing.direction();
                        let loc = map.map.grid_pos(vec).unwrap_or_else(|| {
                            panic!(
                                "Ran off the edge from {} to {vec}, facing {:?}",
                                prev.loc.as_vec(&map.map),
                                prev.facing
                            )
                        });
                        Some(State {
                            loc,
                            facing: prev.facing,
                        })
                    })
                },
            )
            .tuple_windows()
            .flat_map(|(a, b)| {
                if matches!(map.map.get(b.loc).unwrap(), Tile::Solid) {
                    Either::Left(std::iter::repeat(a))
                } else {
                    Either::Right(std::iter::once(a))
                }
            })
            .nth(dist)
            .unwrap(),
        }
    }

    fn value(&self, map: &GridMap<Tile>) -> u64 {
        let (x, y) = self.loc.as_xy(map);
        let x = x as u64;
        let y = y as u64;
        1000 * (y + 1) + 4 * (x + 1) + self.facing.value()
    }
}

impl MonkeyMap {
    fn initial_state(&self) -> State {
        let loc = self
            .map
            .iter_ray(self.map.top_left(), (1, 0))
            .find_map(|(pos, tile)| matches!(tile, Tile::Open).then(|| pos))
            .unwrap();
        State {
            loc,
            facing: Facing::Right,
        }
    }

    fn with_wrapping_teleports(&self) -> Self {
        let teleports: HashMap<State, State> = self
            .map
            .iter()
            .filter_map(|(pos, tile)| {
                (!matches!(tile, Tile::Wrap)).then(|| pos)
            })
            .flat_map(|loc| {
                Facing::iter().map(move |facing| State { loc, facing })
            })
            .filter(|state| {
                let next =
                    state.loc.as_vec(&self.map) + state.facing.direction();
                let (x, y) = self.map.shape();
                let next = (
                    next.x().rem_euclid(x as i64),
                    next.y().rem_euclid(y as i64),
                );
                if let Some(tile) = self.map.get(next) {
                    matches!(tile, Tile::Wrap)
                } else {
                    panic!("Can't find {:?}", next)
                }
            })
            .map(|state| {
                let loc = std::iter::successors(
                    Some(state.loc.as_vec(&self.map)),
                    |prev| {
                        let next = *prev + state.facing.direction();
                        let (x, y) = self.map.shape();
                        Some(
                            (
                                next.x().rem_euclid(x as i64),
                                next.y().rem_euclid(y as i64),
                            )
                                .into(),
                        )
                    },
                )
                .skip(1)
                .find(|loc| !matches!(self.map[*loc], Tile::Wrap))
                .unwrap();

                let facing = state.facing;
                (
                    state,
                    State {
                        loc: self.map.grid_pos(loc).unwrap(),
                        facing,
                    },
                )
            })
            .collect();

        Self {
            map: self.map.clone(),
            teleports,
        }
    }

    fn with_cube_teleports(&self) -> Self {
        let (x, y) = self.map.shape();
        let square_size: i64 = gcd(x as i64, y as i64);

        let cube_teleports: HashMap<State, State> = if square_size == 4 {
            use Facing::*;
            std::iter::empty()
                .chain((0..4).map(|x| ((4 + x, 4, Up), (8, x, Right))))
                .chain((0..4).map(|x| ((11, 4 + x, Right), (15 - x, 8, Down))))
                .chain((0..4).map(|x| ((12 + x, 11, Down), (0, 7 - x, Right))))
                .chain((0..4).map(|x| ((15, 8 + x, Right), (0, 3 - x, Left))))
                .chain((0..4).map(|x| ((0 + x, 7, Down), (11 - x, 11, Up))))
                .flat_map(|(a, b)| {
                    vec![
                        (a, b),
                        ((b.0, b.1, b.2.reverse()), (a.0, a.1, a.2.reverse())),
                    ]
                    .into_iter()
                })
                .map(|((x1, y1, f1), (x2, y2, f2))| {
                    (
                        State {
                            loc: self.map.grid_pos((x1, y1)).unwrap(),
                            facing: f1,
                        },
                        State {
                            loc: self.map.grid_pos((x2, y2)).unwrap(),
                            facing: f2,
                        },
                    )
                })
                .collect()
        } else if square_size == 50 {
            use Facing::*;
            std::iter::empty::<((i64, i64, Facing), (i64, i64, Facing))>()
                .chain(
                    (0..50).map(|x| ((149, 0 + x, Right), (99, 149 - x, Left))),
                )
                .chain(
                    (0..50).map(|x| ((50 + x, 149, Down), (49, 150 + x, Left))),
                )
                .chain(
                    (0..50).map(|x| ((0 + x, 199, Down), (100 + x, 0, Down))),
                )
                .chain(
                    (0..50).map(|x| ((50, 50 + x, Left), (0 + x, 100, Down))),
                )
                .chain(
                    (0..50).map(|x| ((50, 0 + x, Left), (0, 149 - x, Right))),
                )
                .chain((0..50).map(|x| ((50 + x, 0, Up), (0, 150 + x, Right))))
                .chain(
                    (0..50).map(|x| ((99, 50 + x, Right), (100 + x, 49, Up))),
                )
                .flat_map(|(a, b)| {
                    vec![
                        (a, b),
                        ((b.0, b.1, b.2.reverse()), (a.0, a.1, a.2.reverse())),
                    ]
                    .into_iter()
                })
                .map(|((x1, y1, f1), (x2, y2, f2))| {
                    (
                        State {
                            loc: self.map.grid_pos((x1, y1)).unwrap(),
                            facing: f1,
                        },
                        State {
                            loc: self.map.grid_pos((x2, y2)).unwrap(),
                            facing: f2,
                        },
                    )
                })
                .collect()
        } else {
            panic!("Ran into limits of my lazy hard-coded cube folding");
        };

        Self {
            map: self.map.clone(),
            teleports: cube_teleports,
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tile::Open => write!(f, "."),
            Tile::Solid => write!(f, "#"),
            Tile::Wrap => write!(f, " "),
        }
    }
}

impl FromStr for Tile {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = s.chars().exactly_one()?;
        match c {
            '.' => Ok(Tile::Open),
            '#' => Ok(Tile::Solid),
            ' ' => Ok(Tile::Wrap),
            _ => Err(Error::UnknownChar(c)),
        }
    }
}

pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;
    const YEAR: u32 = 2022;
    const DAY: u8 = 22;

    type ParsedInput = (MonkeyMap, Vec<Command>);
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let map_lines: Vec<_> =
            lines.by_ref().take_while(|line| !line.is_empty()).collect();
        let map_line_longest =
            map_lines.iter().map(|line| line.len()).max().unwrap();
        let map = map_lines
            .into_iter()
            .enumerate()
            .flat_map(|(i, line)| {
                line.chars()
                    .chain(
                        std::iter::repeat(' ')
                            .take(map_line_longest - line.len()),
                    )
                    .enumerate()
                    .map(move |(j, c)| {
                        (j, i, c.to_string().parse::<Tile>().unwrap())
                    })
            })
            .collect();
        let map = MonkeyMap {
            map,
            teleports: HashMap::new(),
        };

        let commands = lines
            .exactly_one()?
            .chars()
            .group_by(|c| match c {
                'L' => Some(1),
                'R' => Some(2),
                '0'..='9' => Some(3),
                _ => None,
            })
            .into_iter()
            .map(|(ty, mut chars)| {
                let first = chars.next().unwrap();
                match (ty, first) {
                    (None, _) => Err(Error::UnknownChar(first)),
                    (Some(1), 'L') => Ok(Command::TurnLeft),
                    (Some(2), 'R') => Ok(Command::TurnRight),
                    (Some(3), _) => Ok(Command::Advance(
                        std::iter::once(first)
                            .chain(chars)
                            .collect::<String>()
                            .parse()?,
                    )),
                    _ => Err(Error::UnknownChar(first)),
                }
            })
            .collect::<Result<_, _>>()?;
        Ok((map, commands))
    }

    type Part1Result = u64;
    fn part_1(
        (map, commands): &Self::ParsedInput,
    ) -> Result<Self::Part1Result, Error> {
        let map = map.with_wrapping_teleports();

        let final_state: State = commands
            .iter()
            .cloned()
            .scan(map.initial_state(), |state, command| {
                *state = state.apply(&map, command);
                Some(state.clone())
            })
            .last()
            .unwrap();

        Ok(final_state.value(&map.map))
    }

    type Part2Result = u64;
    fn part_2(
        (map, commands): &Self::ParsedInput,
    ) -> Result<Self::Part2Result, Error> {
        let map = map.with_cube_teleports();

        let final_state: State = commands
            .iter()
            .cloned()
            .scan(map.initial_state(), |state, command| {
                *state = state.apply(&map, command);
                Some(state.clone())
            })
            .last()
            .unwrap();

        Ok(final_state.value(&map.map))
    }
}
