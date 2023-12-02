use aoc_utils::prelude::*;

use std::collections::VecDeque;

struct Data {
    numbers: VecDeque<Number>,
}

struct Number {
    original_index: usize,
    value: i64,
}

impl Data {
    fn new(data: &[i64], decryption_key: i64) -> Self {
        let numbers = data
            .iter()
            .enumerate()
            .map(|(i, &value)| Number {
                original_index: i,
                value: value * decryption_key,
            })
            .collect();
        Self { numbers }
    }

    fn mix(&mut self) {
        let n = self.numbers.len();
        for old_index in 0..n {
            let current_index: usize = self
                .numbers
                .iter()
                .enumerate()
                .find_map(|(new_index, number)| {
                    (old_index == number.original_index).then_some(new_index)
                })
                .unwrap();

            self.numbers.rotate_left(current_index);
            let number = self.numbers.pop_front().unwrap();
            self.numbers
                .rotate_left(number.value.rem_euclid((n - 1) as i64) as usize);
            self.numbers.push_front(number);
        }
    }

    fn coordinates(&self) -> i64 {
        let n = self.numbers.len();
        let zero_index: usize = self
            .numbers
            .iter()
            .enumerate()
            .find_map(|(i, number)| (number.value == 0).then_some(i))
            .unwrap();

        [1000, 2000, 3000]
            .iter()
            .map(|i| *i as usize)
            .map(|index| (zero_index + index).rem_euclid(n))
            .map(|index| self.numbers[index].value)
            .inspect(|val| println!("Value: {val}"))
            .sum()
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<i64>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines.map(|line| line.parse()).collect::<Result<_, _>>()?)
    }

    fn part_1(data: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        let mut mixed = Data::new(data, 1);
        mixed.mix();
        Ok(mixed.coordinates())
    }

    fn part_2(data: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        let decryption_key = 811589153;
        let mut mixed = Data::new(data, decryption_key);
        (0..10).for_each(|_| mixed.mix());
        Ok(mixed.coordinates())
    }
}
