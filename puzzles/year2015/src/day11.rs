use aoc_utils::prelude::*;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

const BASE: u64 = 26;

#[derive(Debug, Clone, Copy)]
pub struct Password(u64);

impl std::str::FromStr for Password {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Password(
            s.chars()
                .map(|c| c.to_digit(36).unwrap())
                .fold(0u64, |prev, digit| BASE * prev + ((digit - 10) as u64)),
        ))
    }
}

impl std::fmt::Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.iter_char().try_for_each(|c| write!(f, "{c}"))
    }
}

impl Password {
    fn max() -> Self {
        Self(BASE.pow(8))
    }

    fn iter_digit(self) -> impl Iterator<Item = u8> {
        (0..8).rev().map(move |pow| {
            self.0.div_euclid(BASE.pow(pow)).rem_euclid(BASE) as u8
        })
    }

    fn iter_char(self) -> impl Iterator<Item = char> {
        self.iter_digit()
            .map(|val| char::from_digit(10 + (val as u32), 36).unwrap())
    }

    fn has_sequential_triplet(&self) -> bool {
        self.iter_digit()
            .tuple_windows()
            .any(|(a, b, c)| a + 1 == b && a + 2 == c)
    }

    fn has_invalid_char(&self) -> bool {
        self.iter_char().any(|c| matches!(c, 'i' | 'o' | 'l'))
    }

    fn num_unique_pairs(&self) -> usize {
        self.iter_digit()
            .tuple_windows()
            .filter(|(a, b)| a == b)
            .unique()
            .count()
    }

    fn is_valid(&self) -> bool {
        self.has_sequential_triplet()
            && !self.has_invalid_char()
            && (self.num_unique_pairs() >= 2)
    }

    fn next_valid(&self) -> Result<Self, Error> {
        let min = (self.0 + 1) as usize;
        let max = Password::max().0 as usize;

        (min..max)
            .map(|val| Password(val as u64))
            .find(|password| password.is_valid())
            .ok_or(Error::EarlyFailure)
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Password;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).exactly_one_or_err()?
    }

    fn part_1(
        password: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        password.next_valid().map(|password| format!("{password}"))
    }

    fn part_2(
        password: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        password
            .next_valid()
            .and_then(|pw| pw.next_valid())
            .map(|password| format!("{password}"))
    }
}
