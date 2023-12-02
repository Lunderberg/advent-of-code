use aoc_utils::prelude::*;

#[derive(Debug)]
pub struct BoxSize(i64, i64, i64);

impl std::str::FromStr for BoxSize {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        s.split('x')
            .map(|val| -> Result<i64, _> { val.parse() })
            .tuples()
            .map(|(x, y, z)| -> Result<_, Error> { Ok(BoxSize(x?, y?, z?)) })
            .exactly_one_or_err()?
    }
}

impl BoxSize {
    fn wrapping_paper(&self) -> i64 {
        let BoxSize(x, y, z) = self;
        2 * x * y + 2 * y * z + 2 * x * z + x * y * z / x.max(y).max(z)
    }

    fn ribbon(&self) -> i64 {
        let BoxSize(x, y, z) = self;

        2 * (x + y + z - x.max(y).max(z)) + x * y * z
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<BoxSize>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = values.iter().map(|b| b.wrapping_paper()).sum::<i64>();
        Ok(value)
    }

    fn part_2(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = values.iter().map(|b| b.ribbon()).sum::<i64>();
        Ok(value)
    }
}
