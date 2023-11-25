#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl ThisDay {
    fn get_bit_mask(entries: &[Vec<bool>]) -> Result<usize, Error> {
        let num_bits = entries
            .iter()
            .map(|entry| entry.len())
            .max()
            .ok_or(Error::NoneError)?;
        Ok((1 << num_bits) - 1)
    }

    fn filter_most_frequent(
        entries: &[Vec<bool>],
        reverse_filter: bool,
    ) -> Result<usize, Error> {
        let mut remaining: Vec<Option<&Vec<bool>>> =
            entries.iter().map(Some).collect();
        let num_bits = entries
            .iter()
            .map(|entry| entry.len())
            .max()
            .ok_or(Error::NoneError)?;

        (0..num_bits).for_each(|i_bit| {
            let num_remaining = remaining.iter().flatten().count();
            if num_remaining > 1 {
                let num_true = remaining
                    .iter()
                    .flatten()
                    .map(|entry| entry[i_bit] as usize)
                    .sum::<usize>();
                let num_false = num_remaining - num_true;

                let val_to_keep = (num_true >= num_false) ^ reverse_filter;

                remaining = remaining
                    .iter()
                    .map(|entry| {
                        entry
                            .map(|vals| {
                                if vals[i_bit] == val_to_keep {
                                    Some(vals)
                                } else {
                                    None
                                }
                            })
                            .flatten()
                    })
                    .collect();
                if num_true > num_remaining / 2 {}
            }
        });

        let result = remaining.into_iter().flatten().exactly_one()?;

        Ok(result.iter().fold(0, |acc, &b| 2 * acc + (b as usize)))
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<Vec<bool>>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines
            .map(|line| {
                line.chars()
                    .map(move |c| match c {
                        '0' => Ok(false),
                        '1' => Ok(true),
                        _ => Err(Error::InvalidArg(line.into())),
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()
    }

    fn part_1(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let bit_mask = Self::get_bit_mask(parsed)?;
        let gamma = parsed
            .iter()
            .flat_map(|entry: &Vec<bool>| entry.iter().enumerate())
            .into_group_map()
            .into_iter()
            .sorted_by_key(|(bitnum, _vals)| bitnum.to_owned())
            .map(|(_bitnum, vals)| {
                let is_set = vals.iter().map(|&b| *b as usize).sum::<usize>()
                    > vals.len() / 2;
                is_set
            })
            .fold(0, |acc, b| 2 * acc + (b as usize));
        let epsilon = (!gamma) & bit_mask;
        Ok(gamma * epsilon)
    }

    fn part_2(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let oxy = Self::filter_most_frequent(parsed, false)?;
        let carbon_dioxide = Self::filter_most_frequent(parsed, true)?;
        Ok(oxy * carbon_dioxide)
    }
}
