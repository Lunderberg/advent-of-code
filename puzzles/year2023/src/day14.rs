use std::fmt::Display;

use aoc_utils::prelude::*;

#[derive(Clone, PartialEq, Eq)]
pub struct Platform(GridMap<Tile>);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
    RoundRock,
    CubeRock,
    Empty,
}

#[derive(Debug, Copy, Clone)]
enum Direction {
    North,
    West,
    South,
    East,
}

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Tile::RoundRock => 'O',
            Tile::CubeRock => '#',
            Tile::Empty => '.',
        };
        write!(f, "{c}")
    }
}

impl TryFrom<char> for Tile {
    type Error = Error;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            'O' => Ok(Tile::RoundRock),
            '#' => Ok(Tile::CubeRock),
            '.' => Ok(Tile::Empty),
            _ => Err(Error::UnknownChar(c)),
        }
    }
}

impl Direction {
    fn as_vec(&self) -> Vector<2, i64> {
        match self {
            Direction::North => [0, -1].into(),
            Direction::West => [-1, 0].into(),
            Direction::South => [0, 1].into(),
            Direction::East => [1, 0].into(),
        }
    }
}

impl Platform {
    fn after_tilt_direction(mut self, dir: Direction) -> Self {
        let map = &mut self.0;
        let (width, height) = map.shape();
        let width = width as i64;
        let height = height as i64;

        // Need to iterate in direction of increasing slope, so a
        // CubeRock will never fall down to an unsupported CubeRock.
        let pos_iter: Box<dyn Iterator<Item = Vector<2, i64>>> =
            match dir {
                Direction::North => Box::new(
                    (0..width)
                        .flat_map(|x| (0..height).map(move |y| [x, y].into())),
                ),
                Direction::West => Box::new(
                    (0..height)
                        .flat_map(|y| (0..width).map(move |x| [x, y].into())),
                ),
                Direction::South => Box::new((0..width).flat_map(|x| {
                    (0..height).rev().map(move |y| [x, y].into())
                })),
                Direction::East => Box::new((0..height).flat_map(|y| {
                    (0..width).rev().map(move |x| [x, y].into())
                })),
            };

        for pos in pos_iter {
            if map[pos] == Tile::RoundRock {
                let falls_to = (1..)
                    .map(|steps| pos + steps * dir.as_vec())
                    .find(|&underneath_pos| {
                        !matches!(map.get(underneath_pos), Some(Tile::Empty))
                    })
                    .map(|underneath_pos| underneath_pos - dir.as_vec())
                    .expect(
                        "Loop should terminate at edge of map, at the latest",
                    );

                if falls_to != pos {
                    map[falls_to] = Tile::RoundRock;
                    map[pos] = Tile::Empty;
                }
            }
        }

        self
    }

    fn after_cycle(self) -> Self {
        [
            Direction::North,
            Direction::West,
            Direction::South,
            Direction::East,
        ]
        .into_iter()
        .fold(self, |a, dir| a.after_tilt_direction(dir))
    }

    // fn tilt_north(&self) -> Self {
    //     let mut out = self.clone();
    //     let map = &mut out.0;

    //     let (width, height) = map.shape();
    //     let width = width as i64;
    //     let height = height as i64;

    //     for i in 0..width {
    //         for j in 0..height {
    //             if map[(i, j)] == Tile::RoundRock {
    //                 let falls_to = (0..j)
    //                     .rev()
    //                     .find(|j2| map[(i, *j2)] != Tile::Empty)
    //                     .map(|j2| j2 + 1)
    //                     .unwrap_or(0);
    //                 if falls_to != j {
    //                     map[(i, falls_to)] = Tile::RoundRock;
    //                     map[(i, j)] = Tile::Empty;
    //                 }
    //             }
    //         }
    //     }

    //     out
    // }

    fn load_on_north(&self) -> i64 {
        let (_, height) = self.0.shape();
        let height = height as i64;

        self.0
            .iter()
            .filter(|(_, tile): &((i64, i64), &Tile)| {
                matches!(tile, Tile::RoundRock)
            })
            .map(|((_, j), _)| height - j)
            .sum::<i64>()
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Platform;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(Platform(lines.collect()))
    }

    fn part_1(
        platform: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let platform = platform.clone().after_tilt_direction(Direction::North);
        Ok(platform.load_on_north())
    }

    fn part_2(
        platform: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let num_cycles = 1000000000;

        let (num_iter, fixed_point) = std::iter::successors(
            Some((platform.clone(), platform.clone())),
            |(prev_slow, prev_fast)| {
                let next_slow = prev_slow.clone().after_cycle();
                let next_fast = prev_fast.clone().after_cycle().after_cycle();

                Some((next_slow, next_fast))
            },
        )
        .enumerate()
        .take_while_inclusive(|(i, (slow, fast))| *i == 0 || slow != fast)
        .last()
        .map(|(i, (slow, _))| (i, slow))
        .unwrap();

        let after_remainder = (0..(num_cycles % num_iter))
            .fold(fixed_point, |state, _| state.clone().after_cycle());

        Ok(after_remainder.load_on_north())
    }
}
