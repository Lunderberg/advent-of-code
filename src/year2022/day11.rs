#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
pub struct KeepAway {
    monkeys: Vec<Monkey>,
    items: Vec<ExactItem>,
}

#[derive(Debug, Clone)]
struct ExactItem {
    worry: i64,
    held_by: usize,
    round: usize,
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    Mul(i64),
    Add(i64),
    Square,
}

#[derive(Debug, Clone)]
struct Monkey {
    operation: Operation,
    divisible_by: i64,
    target_true: usize,
    target_false: usize,
}

impl Operation {
    fn apply(&self, val: i64) -> i64 {
        match self {
            Operation::Mul(rhs) => val * rhs,
            Operation::Add(rhs) => val + rhs,
            Operation::Square => val * val,
        }
    }
}

impl KeepAway {
    fn track_item_part1(
        &self,
        item: &ExactItem,
    ) -> impl Iterator<Item = ExactItem> + '_ {
        std::iter::successors(Some(item.clone()), move |prev| {
            let monkey = &self.monkeys[prev.held_by];
            let worry = monkey.operation.apply(prev.worry) / 3;
            let held_by = if worry % monkey.divisible_by == 0 {
                monkey.target_true
            } else {
                monkey.target_false
            };
            let round = if held_by > prev.held_by {
                prev.round
            } else {
                prev.round + 1
            };
            Some(ExactItem {
                worry,
                held_by,
                round,
            })
        })
    }

    /// Track the path of a single item as it passes among monkeys.
    /// Does not include the "worry /= 3" of part a.
    fn track_item_part2(
        &self,
        item: &ExactItem,
    ) -> impl Iterator<Item = ExactItem> + '_ {
        let lcm: i64 = self
            .monkeys
            .iter()
            .map(|monkey| monkey.divisible_by)
            .product();
        std::iter::successors(Some(item.clone()), move |prev| {
            let monkey = &self.monkeys[prev.held_by];
            let worry = monkey.operation.apply(prev.worry) % lcm;
            let held_by = if worry % monkey.divisible_by == 0 {
                monkey.target_true
            } else {
                monkey.target_false
            };
            let round = if held_by > prev.held_by {
                prev.round
            } else {
                prev.round + 1
            };
            Some(ExactItem {
                worry,
                held_by,
                round,
            })
        })
    }
}

pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;
    const YEAR: u32 = 2022;
    const DAY: u8 = 11;

    type ParsedInput = KeepAway;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let (monkeys, items_by_monkey): (Vec<_>, Vec<_>) = lines
            .chunks(7)
            .into_iter()
            .map(|mut chunk| -> Result<(Monkey, Vec<ExactItem>), Error> {
                let id = {
                    let line =
                        chunk.next().ok_or(Error::UnexpectedEndOfStream)?;
                    line.strip_prefix("Monkey ")
                        .and_then(|s| s.strip_suffix(":"))
                        .ok_or_else(|| Error::InvalidString(line.to_string()))?
                        .parse()?
                };
                let items: Vec<ExactItem> = {
                    let line =
                        chunk.next().ok_or(Error::UnexpectedEndOfStream)?;
                    line.strip_prefix("  Starting items: ")
                        .ok_or_else(|| Error::InvalidString(line.to_string()))?
                        .split(", ")
                        .map(|s| Ok(s.parse::<i64>()?))
                        .collect::<Result<Vec<_>, Error>>()?
                        .into_iter()
                        .map(|worry| ExactItem {
                            worry,
                            held_by: id,
                            round: 0,
                        })
                        .collect()
                };
                let operation = {
                    let line =
                        chunk.next().ok_or(Error::UnexpectedEndOfStream)?;
                    line.strip_prefix("  Operation: new = old ")
                        .ok_or_else(|| Error::InvalidString(line.to_string()))
                        .and_then(|op_str| {
                            let (op, arg) =
                                op_str.split_once(" ").ok_or_else(|| {
                                    Error::InvalidString(line.to_string())
                                })?;
                            match (op, arg) {
                                ("*", "old") => Ok(Operation::Square),
                                ("+", arg) => Ok(Operation::Add(arg.parse()?)),
                                ("*", arg) => Ok(Operation::Mul(arg.parse()?)),
                                _ => {
                                    Err(Error::InvalidString(line.to_string()))
                                }
                            }
                        })?
                };
                let divisible_by = {
                    let line =
                        chunk.next().ok_or(Error::UnexpectedEndOfStream)?;
                    line.strip_prefix("  Test: divisible by ")
                        .ok_or_else(|| Error::InvalidString(line.to_string()))?
                        .parse()?
                };
                let target_true = {
                    let line =
                        chunk.next().ok_or(Error::UnexpectedEndOfStream)?;
                    line.strip_prefix("    If true: throw to monkey ")
                        .ok_or_else(|| Error::InvalidString(line.to_string()))?
                        .parse()?
                };
                let target_false = {
                    let line =
                        chunk.next().ok_or(Error::UnexpectedEndOfStream)?;
                    line.strip_prefix("    If false: throw to monkey ")
                        .ok_or_else(|| Error::InvalidString(line.to_string()))?
                        .parse()?
                };

                Ok((
                    Monkey {
                        operation,
                        divisible_by,
                        target_true,
                        target_false,
                    },
                    items,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();

        let items = items_by_monkey
            .into_iter()
            .flat_map(|s| s.into_iter())
            .collect();

        Ok(KeepAway { monkeys, items })
    }

    type Part1Result = usize;
    fn part_1(
        keepaway: &Self::ParsedInput,
    ) -> Result<Self::Part1Result, Error> {
        Ok(keepaway
            .items
            .iter()
            .flat_map(|item| {
                keepaway
                    .track_item_part1(item)
                    .take_while(|item| item.round < 20)
            })
            .map(|item| item.held_by)
            .counts()
            .into_values()
            .sorted_by_key(|val| std::cmp::Reverse(*val))
            .take(2)
            .product::<usize>())
    }

    type Part2Result = usize;
    fn part_2(
        keepaway: &Self::ParsedInput,
    ) -> Result<Self::Part2Result, Error> {
        Ok(keepaway
            .items
            .iter()
            .flat_map(|item| {
                keepaway
                    .track_item_part2(item)
                    .take_while(|item| item.round < 10000)
            })
            .map(|item| item.held_by)
            .counts()
            .into_values()
            .sorted_by_key(|val| std::cmp::Reverse(*val))
            .take(2)
            .product::<usize>())
    }
}
