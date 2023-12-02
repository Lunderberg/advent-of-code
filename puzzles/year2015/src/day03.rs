use aoc_utils::prelude::*;

pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn as_offset(&self) -> Vector<2> {
        match self {
            Direction::North => [0, 1].into(),
            Direction::East => [1, 0].into(),
            Direction::South => [0, -1].into(),
            Direction::West => [-1, 0].into(),
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
                '<' => Ok(Direction::West),
                '>' => Ok(Direction::East),
                'v' => Ok(Direction::South),
                '^' => Ok(Direction::North),
                _ => Err(Error::UnknownChar(c)),
            })
            .collect()
    }

    fn part_1(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let new_houses = values.iter().map(|dir| dir.as_offset()).scan(
            Vector::zero(),
            |state, delta| {
                *state = *state + delta;
                Some(*state)
            },
        );
        let value = std::iter::once(Vector::zero())
            .chain(new_houses)
            .unique()
            .count();
        Ok(value)
    }

    fn part_2(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let new_houses = values
            .iter()
            .map(|dir| dir.as_offset())
            .tuples()
            .scan(
                (Vector::zero(), Vector::zero()),
                |state, (delta_a, delta_b)| {
                    let (a, b) = *state;
                    let a = a + delta_a;
                    let b = b + delta_b;
                    *state = (a, b);
                    Some((a, b))
                },
            )
            .flat_map(|(a, b)| [a, b].into_iter());
        let value = std::iter::once(Vector::zero())
            .chain(new_houses)
            .unique()
            .count();
        Ok(value)
    }
}
