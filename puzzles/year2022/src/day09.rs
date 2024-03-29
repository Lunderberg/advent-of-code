use aoc_utils::prelude::*;

use std::collections::HashSet;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl From<Direction> for Vector<2> {
    fn from(val: Direction) -> Self {
        match val {
            Direction::Up => Vector::new([0, 1]),
            Direction::Down => Vector::new([0, -1]),
            Direction::Left => Vector::new([-1, 0]),
            Direction::Right => Vector::new([1, 0]),
        }
    }
}

#[derive(Debug)]
pub struct Command {
    direction: Direction,
    distance: usize,
}

impl std::str::FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (dir, dist) = s.split_whitespace().tuples().exactly_one_or_err()?;
        let dir = match dir {
            "U" => Ok(Direction::Up),
            "D" => Ok(Direction::Down),
            "L" => Ok(Direction::Left),
            "R" => Ok(Direction::Right),
            _ => Err(Error::InvalidString(dir.to_string())),
        }?;
        let dist = dist.parse()?;
        Ok(Self {
            direction: dir,
            distance: dist,
        })
    }
}

#[derive(Debug, Clone)]
struct Rope {
    locations: Vec<Vector<2>>,
}

impl Rope {
    fn new(n: usize) -> Self {
        Self {
            locations: vec![Vector::zero(); n],
        }
    }

    fn tail(&self) -> Vector<2> {
        *self.locations.last().unwrap()
    }

    fn offsets(&self, dir: Direction) -> impl Iterator<Item = Vector<2>> + '_ {
        let first: Vector<2> = dir.into();
        std::iter::once(first).chain(self.locations.windows(2).scan(
            first,
            |prev_offset, window| {
                let head = window[0] + *prev_offset;
                let prev_tail = window[1];
                let tail_dist = (prev_tail - head)
                    .map(|x| x.abs())
                    .iter()
                    .copied()
                    .max()
                    .unwrap();
                let tail_delta = if tail_dist < 2 {
                    Vector::zero()
                } else {
                    (head - prev_tail).map(|delta| delta.signum())
                };
                *prev_offset = tail_delta;
                Some(tail_delta)
            },
        ))
    }

    fn after_move(&self, dir: Direction) -> Self {
        let locations = self
            .locations
            .iter()
            .zip_eq(self.offsets(dir))
            .map(|(loc, offset)| *loc + offset)
            .collect();
        Self { locations }
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 3;

    type ParsedInput = Vec<Command>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(values
            .iter()
            .flat_map(|command| {
                std::iter::repeat(command.direction).take(command.distance)
            })
            .scan(Rope::new(2), |state, direction| {
                *state = state.after_move(direction);
                Some(state.clone())
            })
            .map(|rope| rope.tail())
            .collect::<HashSet<_>>()
            .len())
    }

    fn part_2(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(values
            .iter()
            .flat_map(|command| {
                std::iter::repeat(command.direction).take(command.distance)
            })
            .scan(Rope::new(10), |state, direction| {
                *state = state.after_move(direction);
                Some(state.clone())
            })
            .map(|rope| rope.tail())
            .collect::<HashSet<_>>()
            .len())
    }
}
