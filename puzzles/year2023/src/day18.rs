use aoc_utils::direction::Direction;
use aoc_utils::prelude::*;

use std::str::FromStr;

#[derive(Debug)]
pub struct Command {
    dir: Direction,
    distance: u32,
    color: (u8, u8, u8),
}

impl FromStr for Command {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (dir, distance, color) = s
            .split_ascii_whitespace()
            .collect_tuple()
            .ok_or(Error::WrongIteratorSize)?;

        let dir = match dir {
            "R" => Ok(Direction::Right),
            "L" => Ok(Direction::Left),
            "U" => Ok(Direction::Up),
            "D" => Ok(Direction::Down),
            _ => Err(Error::InvalidString(dir.to_string())),
        }?;
        let distance = distance.parse()?;

        let color = color
            .chars()
            .filter_map(|c| c.to_digit(16))
            .tuples()
            .map(|(a, b)| (a * 16 + b) as u8)
            .collect_tuple()
            .ok_or(Error::WrongIteratorSize)?;

        Ok(Self {
            dir,
            distance,
            color,
        })
    }
}

impl Command {
    fn unpack_color(&self) -> Self {
        let (r, g, b) = self.color;
        let r = r as u32;
        let g = g as u32;
        let b = b as u32;
        let distance = (r * 256 + g) * 16 + b.div_euclid(16);
        let dir = match b.rem_euclid(16) {
            0 => Direction::Right,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Up,
            _ => panic!("Invalid last u8: {b}"),
        };
        Self {
            dir,
            distance,
            color: (0, 0, 0),
        }
    }
}

// TODO: Upstream this to itertools.  Discussion on
// https://github.com/rust-itertools/itertools/pull/350 looks like
// this was a potential implementation, but wasn't used as the
// ExactSizedIterator wasn't considered too onerous.  For me, I wan't
// to avoid requiring that the iterator itself be cloneable.
trait MyCircularTupleWindowExt: Iterator {
    fn my_circular_tuple_windows<T>(self) -> impl Iterator<Item = T>
    where
        T: itertools::traits::HomogeneousTuple<Item = Self::Item>,
        T: Clone;
}
impl<Iter> MyCircularTupleWindowExt for Iter
where
    Iter: Iterator,
    Iter::Item: Clone,
{
    fn my_circular_tuple_windows<T>(mut self) -> impl Iterator<Item = T>
    where
        T: itertools::traits::HomogeneousTuple<Item = Self::Item>,
        T: Clone,
    {
        let head: Vec<_> = self.by_ref().take(3).collect();

        std::iter::empty()
            .chain(head.clone().into_iter())
            .chain(self)
            .chain(head.into_iter())
            .tuple_windows()
    }
}

fn num_tiles_contained(commands: &[Command]) -> i64 {
    commands
        .iter()
        .circular_tuple_windows()
        .filter(|(a, b)| a.dir == b.dir)
        .for_each(|(a, b)| {
            panic!("Sequential commands {a:?} and {b:?} with same direction")
        });

    commands
        .iter()
        .circular_tuple_windows()
        .filter(|(a, b)| a.dir == b.dir.reverse())
        .for_each(|(a, b)| {
            panic!(
                "Sequential commands {a:?} and {b:?} with opposite directionsq"
            )
        });

    let vertical_lines: Vec<_> = commands
        .iter()
        .scan(Vector::zero(), |pos, command| {
            let Command { distance, dir, .. } = command;
            let distance = *distance as i64;
            let dir = *dir;

            *pos += distance * dir.as_vec();

            Some(*pos)
        })
        .my_circular_tuple_windows()
        .filter(|(_, b, c, _)| b.x() == c.x())
        .flat_map(|(a, b, c, d)| {
            let (left_b, right_b) = if a.x() < b.x() && b.y() < c.y() {
                (b + (0, 1).into(), b + (1, 0).into())
            } else if a.x() < b.x() && c.y() < b.y() {
                (b + (0, 0).into(), b + (1, 1).into())
            } else if b.x() < a.x() && b.y() < c.y() {
                (b + (0, 0).into(), b + (1, 1).into())
            } else if b.x() < a.x() && c.y() < b.y() {
                (b + (0, 1).into(), b + (1, 0).into())
            } else {
                panic!("Should be unreachable, for a={a}, b={b}, c={c}")
            };

            let (left_c, right_c) = if b.y() < c.y() && c.x() < d.x() {
                (c + (0, 1).into(), c + (1, 0).into())
            } else if b.y() < c.y() && d.x() < c.x() {
                (c + (0, 0).into(), c + (1, 1).into())
            } else if c.y() < b.y() && c.x() < d.x() {
                (c + (0, 0).into(), c + (1, 1).into())
            } else if c.y() < b.y() && d.x() < c.x() {
                (c + (0, 1).into(), c + (1, 0).into())
            } else {
                panic!("Should be unreachable, for b={b}, c={c}, d={d}")
            };

            [
                (left_b.x(), left_b.y(), left_c.y()),
                (right_b.x(), right_b.y(), right_c.y()),
            ]
            .into_iter()
        })
        .sorted_by_key(|(x, _, _)| *x)
        .collect();

    let (ymin, ymax) = vertical_lines
        .iter()
        .flat_map(|(_, y1, y2)| [*y1, *y2])
        .fold((0, 0), |(ymin, ymax), y| (ymin.min(y), ymax.max(y)));

    let lagoon_size = (ymin..=ymax)
        .map(|y| -> i64 {
            vertical_lines
                .iter()
                .filter(|(_, y1, y2)| y1.min(y2) <= &y && &y < y1.max(y2))
                .map(|(x, y1, y2)| (x, (y2 - y1).signum()))
                .scan(0, |state, (x, sign)| {
                    *state += sign;
                    Some((x, *state))
                })
                .tuple_windows()
                .filter(|((_, offset), (_, _))| *offset != 0)
                .map(|((x1, _), (x2, _))| x2 - x1)
                .sum()
        })
        .sum::<i64>();
    lagoon_size
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<Command>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        commands: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(num_tiles_contained(commands))
    }

    fn part_2(
        commands: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let commands: Vec<_> = commands
            .iter()
            .map(|command| command.unpack_color())
            .collect();
        Ok(num_tiles_contained(&commands))
    }
}
