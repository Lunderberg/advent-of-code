use aoc_utils::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
}

impl Direction {
    fn delta_z(&self) -> i64 {
        match self {
            Direction::Up => 1,
            Direction::Down => -1,
        }
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<Direction>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines
            .flat_map(|line| line.chars())
            .map(|c| match c {
                '(' => Ok(Direction::Up),
                ')' => Ok(Direction::Down),
                _ => Err(Error::UnknownChar(c)),
            })
            .collect()
    }

    fn part_1(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(values.iter().map(|dir| dir.delta_z()).sum::<i64>())
    }

    fn part_2(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(values
            .iter()
            .map(|dir| dir.delta_z())
            .scan(0, |acc, val| {
                *acc += val;
                Some(*acc)
            })
            .enumerate()
            .find(|(_, floor)| *floor < 0)
            .map(|(i, _)| i + 1)
            .unwrap())
    }
}
