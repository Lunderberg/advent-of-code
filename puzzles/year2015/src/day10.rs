use aoc_utils::prelude::*;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

trait LookAndSay {
    fn look_and_say(self) -> impl Iterator<Item = u8>;
}
impl<Iter> LookAndSay for Iter
where
    Iter: Iterator<Item = u8>,
{
    fn look_and_say(self) -> impl Iterator<Item = u8> {
        let mut iter = self.peekable();
        std::iter::from_fn(move || {
            let digit = iter.next()?;
            let mut count = 1;

            while let Some(_) = iter.next_if(|peek| *peek == digit) {
                count += 1;
            }
            assert!(count < 10);
            Some([count, digit].into_iter())
        })
        .flatten()
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<u8>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines
            .flat_map(|line| line.chars())
            .filter_map(|c| c.to_digit(10))
            .map(|digit| digit as u8)
            .collect())
    }

    fn part_1(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let mut iter: Box<dyn Iterator<Item = u8>> =
            Box::new(values.iter().map(|c| *c));
        for _ in 0..40 {
            iter = Box::new(iter.look_and_say());
        }
        Ok(iter.count())
    }

    fn part_2(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let mut iter: Box<dyn Iterator<Item = u8>> =
            Box::new(values.iter().map(|c| *c));
        for _ in 0..50 {
            iter = Box::new(iter.look_and_say());
        }
        Ok(iter.count())
    }
}
