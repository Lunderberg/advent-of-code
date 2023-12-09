use aoc_utils::prelude::*;

#[derive(Debug)]
pub struct Sequence(Vec<i64>);

impl Sequence {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn first(&self) -> Option<i64> {
        self.0.iter().next().cloned()
    }

    fn last(&self) -> Option<i64> {
        self.0.last().cloned()
    }

    fn is_all_zero(&self) -> bool {
        self.0.iter().all(|&val| val == 0)
    }

    fn delta(&self) -> Self {
        Self(self.0.iter().tuple_windows().map(|(a, b)| b - a).collect())
    }

    fn integrate_forward(&self, initial: i64) -> Self {
        Self(
            std::iter::once(initial)
                .chain(self.0.iter().scan(initial, |cumsum, &value| {
                    *cumsum += value;
                    Some(*cumsum)
                }))
                .collect(),
        )
    }

    fn extend_forward(&self) -> Self {
        assert!(!self.0.is_empty());

        if self.is_all_zero() {
            Self(vec![0; self.len() + 1])
        } else {
            self.delta()
                .extend_forward()
                .integrate_forward(self.first().unwrap())
        }
    }

    fn rev(&self) -> Self {
        Self(self.0.iter().rev().cloned().collect())
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<Sequence>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines
            .map(|line| -> Result<Sequence, Error> {
                let values = line
                    .split_ascii_whitespace()
                    .map(|val| val.parse())
                    .collect::<Result<Vec<i64>, _>>()?;
                Ok(Sequence(values))
            })
            .collect()
    }

    fn part_1(
        sequences: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = sequences
            .iter()
            .map(|sequence| sequence.extend_forward().last().unwrap())
            .sum::<i64>();
        Ok(value)
    }

    fn part_2(
        sequences: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = sequences
            .iter()
            .map(|sequence| sequence.rev().extend_forward().last().unwrap())
            .sum::<i64>();
        Ok(value)
    }
}
