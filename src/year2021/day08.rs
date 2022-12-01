#![allow(dead_code)]
#![allow(unused_imports)]
use crate::{Error, Puzzle};

use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::fmt::{Debug, Formatter};

use itertools::Itertools;

pub struct Day08;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
struct Segment(u8);

#[derive(Debug)]
pub struct LightSequence {
    unique_patterns: Vec<HashSet<Segment>>,
    outputs: Vec<HashSet<Segment>>,
}

#[derive(Debug)]
struct SegmentConstraints {
    allowed_values: Vec<HashSet<Segment>>,
}

impl Debug for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u8> for Segment {
    type Error = Error;
    fn try_from(val: u8) -> Result<Self, Error> {
        if val < 7 {
            Ok(Segment(val))
        } else {
            Err(Error::InvalidDigit(val))
        }
    }
}

impl TryFrom<char> for Segment {
    type Error = Error;
    fn try_from(c: char) -> Result<Segment, Error> {
        match c {
            'a' => Ok(Segment(0)),
            'b' => Ok(Segment(1)),
            'c' => Ok(Segment(2)),
            'd' => Ok(Segment(3)),
            'e' => Ok(Segment(4)),
            'f' => Ok(Segment(5)),
            'g' => Ok(Segment(6)),
            _ => Err(Error::UnknownChar(c)),
        }
    }
}

impl std::str::FromStr for Segment {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let char = s.chars().exactly_one()?;
        Err(Error::UnknownChar(char))
    }
}

impl std::str::FromStr for LightSequence {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.lines()
            .exactly_one()?
            .split('|')
            .tuples()
            .map(|(a, b)| -> Result<_, Error> {
                let unpack = |string: &str| {
                    string
                        .split(' ')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| {
                            s.chars()
                                .map(|c| c.try_into())
                                .collect::<Result<_, Error>>()
                        })
                        .collect::<Result<_, Error>>()
                };
                Ok(LightSequence {
                    unique_patterns: unpack(a)?,
                    outputs: unpack(b)?,
                })
            })
            .exactly_one()?
    }
}

impl LightSequence {
    fn decode_outputs(&self) -> Result<Vec<u8>, Error> {
        let mapping = self.find_mapping()?;
        let values = self
            .outputs
            .iter()
            .map(|output_pattern| -> Result<u8, Error> {
                let val = mapping
                    .iter()
                    .enumerate()
                    .filter(|(_digit, known_pattern)| {
                        known_pattern == &output_pattern
                    })
                    .map(|(digit, _known_pattern)| digit)
                    .exactly_one()?;
                Ok(val as u8)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(values)
    }

    fn find_mapping(&self) -> Result<Vec<HashSet<Segment>>, Error> {
        let expected_segment_patterns: Vec<HashSet<Segment>> = (0..=9)
            .map(|d| Ok(active_segments(d)?.collect()))
            .collect::<Result<_, Error>>()?;

        let digit_assignment_groups: Vec<(Vec<u8>, Vec<&HashSet<Segment>>)> =
            self.unique_patterns
                .iter()
                .map(|pattern| (pattern.len(), pattern))
                .into_group_map()
                .into_iter()
                .map(|(num_segments, patterns)| {
                    (
                        expected_segment_patterns
                            .iter()
                            .enumerate()
                            .filter(|(_digit, segments)| {
                                num_segments == segments.len()
                            })
                            .map(|(digit, _segments)| digit as u8)
                            .collect::<Vec<u8>>(),
                        patterns,
                    )
                })
                .sorted_by(|(a_digits, _), (b_digits, _)| {
                    a_digits.len().cmp(&b_digits.len())
                })
                .collect();

        let possible_digit_assignments = digit_assignment_groups
            .iter()
            .map(|(digits, patterns)| {
                digits
                    .iter()
                    .permutations(digits.len())
                    .map(move |ordering| (ordering, patterns))
            })
            .multi_cartesian_product()
            .map(|vec: Vec<(Vec<&u8>, &Vec<&HashSet<Segment>>)>| {
                let digits: Vec<u8> = vec
                    .iter()
                    .flat_map(|(vals, _)| vals.iter())
                    .map(|d| **d)
                    .collect();
                let patterns: Vec<&HashSet<Segment>> = vec
                    .iter()
                    .flat_map(|(_, patterns)| patterns.iter()).copied()
                    .collect();
                (digits, patterns)
            })
            .collect::<Vec<_>>();

        let digit_assignment = possible_digit_assignments
            .iter()
            .filter(|(digits, patterns)| {
                Self::is_valid_assignment(digits, patterns)
            })
            .map(|(digits, patterns)| {
                digits
                    .iter()
                    .zip(patterns.iter())
                    .sorted_by_key(|(d, _pat)| *d)
                    .map(|(_d, pat)| (*pat).clone())
                    .collect::<Vec<_>>()
            })
            .exactly_one()?;

        Ok(digit_assignment)
    }

    fn is_valid_assignment(
        digits: &[u8],
        patterns: &[&HashSet<Segment>],
    ) -> bool {
        let mut constraints = SegmentConstraints::new();
        digits
            .iter()
            .zip(patterns.iter())
            .for_each(|(digit, pattern)| {
                constraints.impose_constraint(*digit, pattern);
            });

        constraints
            .allowed_values
            .iter()
            .all(|vals| vals.len() == 1)
    }

    fn interpret_results(&self, _segment_map: &Vec<Segment>) -> Vec<u8> {
        let reference_digits: Vec<HashSet<Segment>> = (0..=9)
            .map(|d| Ok(active_segments(d)?.collect()))
            .collect::<Result<_, Error>>()
            .unwrap();

        self.outputs
            .iter()
            .map(|segments| {
                reference_digits
                    .iter()
                    .enumerate()
                    .filter(|(_i, reference)| *reference == segments)
                    .map(|(i, _reference)| i as u8)
                    .exactly_one()
                    .unwrap()
            })
            .collect()
    }
}

impl SegmentConstraints {
    fn new() -> Self {
        let all_segments: HashSet<_> =
            (0..7).map(|seg| seg.try_into().unwrap()).collect();
        let allowed_values =
            (0..7).map(|_segnum| all_segments.clone()).collect();
        Self { allowed_values }
    }

    fn impose_constraint(
        &mut self,
        digit: u8,
        lighted_segments: &HashSet<Segment>,
    ) -> bool {
        // Wrong number of segments, means I did something wrong in
        // the parent scope.
        let expected_num_lit = segments_status(digit)
            .iter()
            .map(|&b| b as usize)
            .sum::<usize>();
        if lighted_segments.len() != expected_num_lit {
            panic!();
        }

        // Remove any options that directly contradict the constraint given.
        self.allowed_values
            .iter_mut()
            .zip(segments_status(digit).iter())
            .for_each(|(rewired_seg_options, actual_lit)| {
                rewired_seg_options
                    .iter()
                    .filter(|rewired_seg| {
                        let rewired_lit =
                            lighted_segments.contains(rewired_seg);
                        *actual_lit != rewired_lit
                    }).copied()
                    .collect::<Vec<Segment>>()
                    .into_iter()
                    .for_each(|incorrect_option| {
                        rewired_seg_options.remove(&incorrect_option);
                    });
            });

        // For any segments that are now known, remove it as a
        // possibility from all other segments.
        self.allowed_values
            .iter()
            .enumerate()
            .filter(|(_i, vals)| vals.len() == 1)
            .map(|(i, vals)| (i, *vals.iter().next().unwrap()))
            .collect::<Vec<(usize, Segment)>>()
            .iter()
            .for_each(|(unique_i, unique_val)| {
                self.allowed_values
                    .iter_mut()
                    .enumerate()
                    .filter(|(i, _vals)| i != unique_i)
                    .for_each(|(_i, vals)| {
                        vals.remove(unique_val);
                    })
            });

        // The set of constraints is valid if every segment still has
        // some possible values.
        self.allowed_values.iter().all(|vals| !vals.is_empty())
    }
}

fn segments_status(digit: u8) -> [bool; 7] {
    match digit {
        0 => [true, true, true, false, true, true, true],
        1 => [false, false, true, false, false, true, false],
        2 => [true, false, true, true, true, false, true],
        3 => [true, false, true, true, false, true, true],
        4 => [false, true, true, true, false, true, false],
        5 => [true, true, false, true, false, true, true],
        6 => [true, true, false, true, true, true, true],
        7 => [true, false, true, false, false, true, false],
        8 => [true, true, true, true, true, true, true],
        9 => [true, true, true, true, false, true, true],
        _ => panic!(),
    }
}

fn active_segments(digit: u8) -> Result<impl Iterator<Item = Segment>, Error> {
    Ok(segments_status(digit)
        .iter()
        .enumerate()
        .filter(|(_i, active)| **active)
        .map(|(i, _active)| (i as u8).try_into().unwrap())
        .collect::<Vec<_>>()
        .into_iter())
}

impl Puzzle for Day08 {
    const YEAR: u32 = 2021;
    const DAY: u8 = 8;
    const IMPLEMENTED: bool = true;
    const EXAMPLE_NUM: u8 = 1;

    type ParsedInput = Vec<LightSequence>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines
            .scan(None, |partial: &mut Option<String>, line| {
                if let Some(partial) = partial.take() {
                    Some(partial + line)
                } else if line.ends_with('|') {
                    *partial = Some(line.to_string());
                    None
                } else {
                    Some(line.to_string())
                }
            })
            .map(|s| s.parse())
            .collect()
    }

    type Part1Result = usize;
    fn part_1(parsed: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        Ok(parsed
            .iter()
            .flat_map(|seq| {
                seq.outputs.iter().filter(|set| match set.len() {
                    2 | 3 | 4 | 7 => true,
                    _ => false,
                })
            })
            .count())
    }

    type Part2Result = usize;
    fn part_2(parsed: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        parsed
            .iter()
            .map(|seq| -> Result<_, Error> {
                Ok(seq
                    .decode_outputs()?
                    .into_iter()
                    .fold(0usize, |acc, digit| 10 * acc + (digit as usize)))
            })
            .sum::<Result<usize, _>>()
    }
}

pub trait FilterPermutable: Iterator {
    fn filter_permute<V>(self, k: usize, validity: V) -> FilterPermute<Self, V>
    where
        Self: Sized,
        Self::Item: Clone,
        V: FnMut(Vec<Self::Item>) -> bool,
    {
        FilterPermute::new(self, k, validity)
    }
}

impl<T> FilterPermutable for T where T: Iterator {}

pub struct FilterPermute<I, V>
where
    I: Iterator,
    I::Item: Clone,
    V: FnMut(Vec<I::Item>) -> bool,
{
    items: Vec<I::Item>,
    permutation_iter: itertools::structs::Permutations<std::ops::Range<usize>>,
    validity: V,
    most_recent_success: Vec<usize>,
    most_recent_failure: Option<Vec<usize>>,
}

impl<I, V> FilterPermute<I, V>
where
    I: Iterator,
    I::Item: Clone,
    V: FnMut(Vec<I::Item>) -> bool,
{
    fn new(iter: I, k: usize, validity: V) -> Self {
        let items: Vec<_> = iter.collect();
        let permutation_iter = (0..items.len()).permutations(k);
        Self {
            items,
            permutation_iter,
            validity,
            most_recent_success: Vec::new(),
            most_recent_failure: None,
        }
    }
}

impl<I, V> Iterator for FilterPermute<I, V>
where
    I: Iterator,
    I::Item: Clone,
    V: FnMut(Vec<I::Item>) -> bool,
{
    type Item = Vec<I::Item>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let permutation = self.permutation_iter.next()?;

            let is_subset_of_failure = self
                .most_recent_failure
                .as_ref()
                .map_or(false, |failed_indices| {
                    failed_indices
                        .iter()
                        .enumerate()
                        .all(|(pos, index)| *index == permutation[pos])
                });
            if is_subset_of_failure {
                continue;
            }
            // Once we've moved past subsets of the most recent
            // failure, we never revisit that region, so we don't need
            // to check for it until the next failure.
            self.most_recent_failure = None;

            let first_test_required = self
                .most_recent_success
                .iter()
                .enumerate()
                .filter(|&(pos, index)| *index == permutation[pos])
                .map(|(pos, _index)| pos + 1)
                .next()
                .unwrap_or(0);

            let first_failing_pos = (first_test_required..permutation.len())
                .filter(|&pos| {
                    let items = (0..=pos)
                        .map(|i| self.items[permutation[i]].clone())
                        .collect();
                    !(self.validity)(items)
                })
                .next();

            // If one of the tests failed, mark the location of the
            // most recent success/failure, then move on.
            if let Some(pos) = first_failing_pos {
                self.most_recent_failure = Some(permutation[0..=pos].to_vec());
                self.most_recent_success = permutation[0..pos].to_vec();
                continue;
            }

            // All checks have passed, so we can return this.
            return Some(
                permutation
                    .iter()
                    .map(|&index| self.items[index].clone())
                    .collect(),
            );
        }
    }
}
