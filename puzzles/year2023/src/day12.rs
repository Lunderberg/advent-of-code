use std::{fmt::Display, str::FromStr};

use aoc_utils::prelude::*;

pub struct Record {
    springs: Vec<Option<bool>>,
    groups: Vec<u8>,
}

impl FromStr for Record {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (springs, groups) = s
            .split_ascii_whitespace()
            .collect_tuple()
            .ok_or(Error::InvalidString(s.to_string()))?;

        let springs = springs
            .chars()
            .map(|c| match c {
                '.' => Ok(Some(false)),
                '#' => Ok(Some(true)),
                '?' => Ok(None),
                _ => Err(Error::UnknownChar(c)),
            })
            .collect::<Result<_, _>>()?;

        let groups = groups
            .split(',')
            .map(|val| val.parse())
            .collect::<Result<_, _>>()?;

        Ok(Self { springs, groups })
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.springs.iter().try_for_each(|spring| {
            let c = match spring {
                Some(true) => '#',
                Some(false) => '.',
                None => '?',
            };
            write!(f, "{c}")
        })?;
        write!(f, " ")?;

        self.groups.iter().enumerate().try_for_each(|(i, group)| {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{group}")
        })
    }
}

impl DirectedGraph<(Vec<bool>, u8, u8)> for Record {
    fn connections_from<'a>(
        &'a self,
        (node, num_true, num_false): &'a (Vec<bool>, u8, u8),
    ) -> impl Iterator<Item = (Vec<bool>, u8, u8)> + '_ {
        [true, false]
            .into_iter()
            .filter(|_| node.len() < self.springs.len())
            .filter(|&b| {
                // self.springs[node.len()]
                //     .map(|known| known == *b)
                //     .unwrap_or(true)
                match self.springs[node.len()] {
                    Some(true) => b,
                    Some(false) => !b,
                    None => (b && *num_true > 0) || (!b && *num_false > 0),
                    // None => true,
                    // None => (!b) || (b && *num_true > 0),
                    // None => (b) || ((!b) && *num_false > 0),
                }
            })
            .filter(|b| {
                self.is_valid_start(
                    node.iter().cloned().chain(std::iter::once(*b)),
                )
            })
            .map(move |b| -> (Vec<bool>, u8, u8) {
                let is_new = self.springs[node.len()].is_none();
                let num_true = num_true.saturating_sub((b && is_new) as u8);
                let num_false =
                    num_false.saturating_sub(((!b) && is_new) as u8);

                (
                    node.iter().cloned().chain(std::iter::once(b)).collect(),
                    num_true,
                    num_false,
                )
            })
    }
}

impl Record {
    fn iter_naive(
        &self,
    ) -> impl Iterator<Item = impl Iterator<Item = bool> + '_> {
        let num_unknown = self
            .springs
            .iter()
            .filter(|spring| spring.is_none())
            .count();

        (0..u64::pow(2, num_unknown as u32)).map(|i| self.iter_spring_guess(i))
    }

    fn iter_spring_guess(&self, mut i: u64) -> impl Iterator<Item = bool> + '_ {
        self.springs.iter().map(move |spring| {
            spring.unwrap_or_else(|| {
                let val = i % 2 == 0;
                i = i >> 1;
                val
            })
        })
    }

    fn count_possible(&self) -> usize {
        let n = self.springs.len();
        let mut count = 0;
        let mut state = vec![false; n];

        // let pbar = indicatif::ProgressBar::new(u32::MAX as u64);

        // let mut value = 0;
        // let mut old_state = state.clone();
        loop {
            // value = {
            //     let new_value = state
            //         .iter()
            //         .rev()
            //         .skip(n.saturating_sub(32))
            //         .rev()
            //         .fold(0, |a, b| 2 * a + (*b as u64));
            //     // if new_value < value {
            //     //     println!(
            //     //         "Old state: {}",
            //     //         old_state
            //     //             .iter()
            //     //             .map(|b| if *b { '#' } else { '.' })
            //     //             .join("")
            //     //     );
            //     //     println!(
            //     //         "New state: {}",
            //     //         state
            //     //             .iter()
            //     //             .map(|b| if *b { '#' } else { '.' })
            //     //             .join("")
            //     //     );
            //     // }
            //     // assert!(
            //     //     new_value >= value,
            //     //     "Value changed from {value} to {new_value}"
            //     // );
            //     // old_state = state.clone();
            //     new_value
            // };
            // pbar.set_position(value);
            // println!(
            //     "\tChecking {}",
            //     state.iter().map(|b| if *b { '#' } else { '.' }).join("")
            // );
            if let Some(i_bit) = self.first_impossible_bit(&state) {
                if state[i_bit] {
                    let mut found_false_bit = false;
                    for i in (0..i_bit).rev() {
                        if let Some(known_bit) = self.springs[i] {
                            state[i] = known_bit;
                        } else if state[i] {
                            state[i] = false;
                        } else {
                            state[i] = true;
                            found_false_bit = true;
                            break;
                        }
                    }
                    if !found_false_bit {
                        break;
                    }
                }

                state[i_bit] = !state[i_bit];

                for i in (i_bit + 1)..n {
                    state[i] = self.springs[i].unwrap_or(false);
                }
            } else {
                // println!("\tThis state is legal");
                count += 1;

                let mut found_false_bit = false;
                for i in (0..n).rev() {
                    if let Some(known_bit) = self.springs[i] {
                        state[i] = known_bit;
                    } else if state[i] {
                        state[i] = false;
                    } else {
                        state[i] = true;
                        found_false_bit = true;
                        break;
                    }
                }
                if !found_false_bit {
                    break;
                }
            }
        }

        // pbar.finish();

        count
    }

    fn first_impossible_bit(&self, bits: &[bool]) -> Option<usize> {
        assert_eq!(bits.len(), self.springs.len());

        let mut i_current_group = 0;
        let mut current_streak: Option<u8> = None;

        let total_num_true = self.groups.iter().sum::<u8>();
        let total_num_false = (self.springs.len() as u8) - total_num_true;

        let mut num_true = 0;
        let mut num_false = 0;

        for (i, (bit, spring)) in
            bits.iter().zip(self.springs.iter()).enumerate()
        {
            // If there is a mismatch with a known bit
            if let Some(known) = spring {
                if bit != known {
                    // println!(
                    //     "\t\tFirst impossible bit is {i}, \
                    //      due to mismatch with known"
                    // );
                    return Some(i);
                }
            }

            if *bit {
                num_true += 1;
                if num_true > total_num_true {
                    // println!(
                    //     "\t\tFirst impossible bit is {i}, \
                    //      due to too many true bits."
                    // );
                    return Some(i);
                }
            } else {
                num_false += 1;
                if num_false > total_num_false {
                    // println!(
                    //     "\t\tFirst impossible bit is {i}, \
                    //      due to too many false bits."
                    // );
                    return Some(i);
                }
            }

            if *bit {
                if let Some(current_streak) = &mut current_streak {
                    *current_streak += 1;
                } else {
                    current_streak = Some(1);
                    if i_current_group >= self.groups.len() {
                        // println!(
                        //     "\t\tFirst impossible bit is {i}, \
                        //      due to exceeding number of groups {}",
                        //     self.groups.len()
                        // );
                        return Some(i);
                    }
                }
                if current_streak.unwrap() > self.groups[i_current_group] {
                    // println!(
                    //     "\t\tFirst impossible bit is {i}, \
                    //      due to streak {} \
                    //      exceeding length of group {i_current_group}",
                    //     current_streak.unwrap()
                    // );
                    return Some(i);
                }
            } else {
                if let Some(current_streak) = &current_streak {
                    if current_streak != &self.groups[i_current_group] {
                        // println!(
                        //     "\t\tFirst impossible bit is {i}, \
                        //      due to streak {} ending \
                        //      without correct group length {}",
                        //     current_streak, self.groups[i_current_group]
                        // );
                        return Some(i);
                    }
                    i_current_group += 1;
                }
                current_streak = None;
            }
        }
        None
    }

    fn count_with_early_backtrack(&self) -> (usize, usize) {
        let group_sum = self.groups.iter().sum::<u8>();
        let (known_true, known_false) =
            self.springs
                .iter()
                .fold((0, 0), |(count_t, count_f), spring| match spring {
                    Some(true) => (count_t + 1, count_f),
                    Some(false) => (count_t, count_f + 1),
                    None => (count_t, count_f),
                });

        let mut total_scanned = 0;
        let result = self
            .iter_depth_first([(
                Vec::new(),
                group_sum - known_true,
                (self.springs.len() as u8) - group_sum - known_false,
            )])
            .map(|(seq, _, _)| seq)
            .inspect(|_| {
                total_scanned += 1;
            })
            .filter(|seq| seq.len() == self.springs.len())
            .filter(|seq| self.is_valid_seq(seq.clone()))
            // .inspect(|seq| {
            //     println!(
            //         "\t{}",
            //         seq.iter()
            //             .map(|b| match b {
            //                 true => '.',
            //                 false => '#',
            //             })
            //             .join("")
            //     )
            // })
            .count();
        // println!("\tScanned {total_scanned} states");
        (result, total_scanned)
    }

    fn is_valid_start(
        &self,
        spring_listing: impl IntoIterator<Item = bool>,
    ) -> bool {
        let mut iter = spring_listing.into_iter();

        self.groups.iter().all(|group_size| {
            let first = iter.next();
            if first.is_none() {
                return true;
            }

            let found_group = std::iter::once(first.unwrap())
                .chain(iter.by_ref())
                .skip_while(|b| !b)
                .take_while(|b| *b)
                .count() as u8;
            found_group == *group_size
                || (found_group < *group_size && iter.next().is_none())
        })
    }

    fn is_valid_seq(
        &self,
        spring_listing: impl IntoIterator<Item = bool>,
    ) -> bool {
        spring_listing
            .into_iter()
            .group_sizes()
            .zip_longest(self.groups.iter().cloned())
            .all(|pair| match pair {
                itertools::EitherOrBoth::Both(a, b) => a == b,
                _ => false,
            })
    }

    fn num_arrangements(&self) -> usize {
        self.iter_naive()
            .filter_map(|seq| {
                if self.is_valid_seq(seq) {
                    Some(())
                } else {
                    None
                }
            })
            .count()
    }

    fn unfold(&self) -> Self {
        let springs = [&self.springs; 5]
            .into_iter()
            .with_position()
            .flat_map(|(pos, springs)| {
                let after = match pos {
                    itertools::Position::Last => None,
                    _ => Some(None),
                };
                springs.into_iter().cloned().chain(after)
            })
            .collect();
        let groups = [&self.groups; 5].into_iter().flatten().cloned().collect();
        Self { springs, groups }
    }
}

trait IterBoolExt {
    fn group_sizes(self) -> impl Iterator<Item = u8>;
}

impl<Iter> IterBoolExt for Iter
where
    Iter: IntoIterator<Item = bool>,
{
    fn group_sizes(self) -> impl Iterator<Item = u8> {
        let mut iter = self.into_iter();
        std::iter::from_fn(move || {
            let first = iter.next()?;

            let group_size = std::iter::once(first)
                .chain(iter.by_ref())
                .skip_while(|b| !b)
                .take_while(|b| *b)
                .count() as u8;

            if group_size == 0 {
                None
            } else {
                Some(group_size)
            }

            // Some(group_size)
        })
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 1;

    type ParsedInput = Vec<Record>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        records: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let arrangements = records
            .iter()
            .inspect(|record| println!("{record}"))
            .map(|record| record.count_possible())
            .inspect(|val| println!("\t{val:?}"))
            .sum::<usize>();

        Ok(arrangements)
    }

    fn part_2(
        records: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let arrangements = records
            .iter()
            .map(|record| record.unfold())
            .inspect(|record| println!("{record}"))
            //.map(|record| record.num_arrangements())
            .map(|record| {
                let start = std::time::Instant::now();
                let result = record.count_possible();
                let elapsed = start.elapsed();
                println!("\t{elapsed:?}");
                result
            })
            .inspect(|val| println!("\t{val:?}"))
            .sum::<usize>();

        Ok(arrangements)
    }
}
