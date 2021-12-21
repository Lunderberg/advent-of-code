#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use crate::utils::geometry::{Mat3, Vector3};

use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};

use itertools::Itertools;
use regex::Regex;

pub struct Day19;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct Orientation(Mat3);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct Location(Vector3);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct Displacement(Vector3);

#[derive(Debug, Eq, Clone, Copy)]
struct DisorientedDisplacement(Vector3);

#[derive(Debug, Clone)]
struct Scanner {
    id: i64,
    beacons: Vec<DisorientedDisplacement>,
}

impl std::ops::Add for Displacement {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(other.0 + self.0)
    }
}

impl std::ops::Sub for Displacement {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(other.0 - self.0)
    }
}

impl Orientation {
    fn identity() -> Self {
        Self(Mat3::identity())
    }

    fn iter() -> impl Iterator<Item = Orientation> {
        (0..=2)
            .flat_map(|alpha| {
                let max_beta = match alpha {
                    0 => 0,
                    1 => 3,
                    2 => 0,
                    _ => panic!("Math is broken"),
                };
                (0..=max_beta).map(move |beta| (alpha, beta))
            })
            .flat_map(|(alpha, beta)| {
                (0..=3).map(move |gamma| (alpha, beta, gamma))
            })
            .map(|(alpha, beta, gamma)| {
                Orientation(
                    Mat3::rotate_z().pow(beta)
                        * Mat3::rotate_x().pow(alpha)
                        * Mat3::rotate_z().pow(gamma),
                )
            })
    }
}

impl DisorientedDisplacement {
    fn with_orientation(&self, orientation: &Orientation) -> Displacement {
        Displacement(orientation.0 * self.0)
    }
}

impl Hash for DisorientedDisplacement {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        Orientation::iter()
            .map(|orientation| self.with_orientation(&orientation))
            .sorted_by_key(|disp| disp.0)
            .for_each(|disp| disp.hash(state));
    }
}

impl PartialEq for DisorientedDisplacement {
    fn eq(&self, other: &Self) -> bool {
        let reference = other.with_orientation(&Orientation::identity());
        Orientation::iter()
            .map(|orientation| self.with_orientation(&orientation))
            .any(|v| v == reference)
    }
}

impl DisorientedDisplacement {
    fn relative_key(&self, other: &Self) -> [i64; 3] {
        let mut output = [0; 3];

        (self.0 - other.0)
            .iter()
            .map(|d| d.abs())
            .sorted()
            .zip(output.iter_mut())
            .for_each(|(val, out)| {
                *out = val;
            });

        output
    }
}

impl std::ops::Add for DisorientedDisplacement {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self(other.0 + self.0)
    }
}

impl std::ops::Sub for DisorientedDisplacement {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self(other.0 - self.0)
    }
}

impl Scanner {
    fn from_lines<'a>(
        lines: &mut impl Iterator<Item = &'a str>,
    ) -> Result<Self, Error> {
        let label = lines.next().ok_or(Error::UnexpectedEndOfStream)?;
        let id = Regex::new(
            r"(?x)
              ^---\sscanner\s
              (?P<id>\d+)
              \s---
              $",
        )
        .unwrap()
        .captures(label)
        .map(|cap| cap.name("id").unwrap().as_str().parse::<i64>().unwrap())
        .ok_or_else(|| Error::InvalidString(label.to_string()))?;

        let beacons = lines
            .by_ref()
            .take_while(|line| line.len() > 0)
            .map(|line| line.parse::<Vector3>())
            .map_ok(|vec| DisorientedDisplacement(vec))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { id, beacons })
    }

    fn beacon_offsets(
        &self,
    ) -> impl Iterator<Item = ([i64; 3], (usize, usize))> + '_ {
        self.beacons
            .iter()
            .enumerate()
            .tuple_combinations()
            .map(|((ia, a), (ib, b))| (b.relative_key(a), (ia, ib)))
    }

    fn shared_beacon_mapping(
        &self,
        other: &Scanner,
    ) -> Result<HashMap<usize, usize>, Error> {
        // Make a lookup map from the (a,b,c) offset array into the
        // pair of other.beacons indices that generates that offset.
        let other_offsets: HashMap<_, _> = other.beacon_offsets().collect();

        let mapping: HashMap<_, _> = self
            // Identify pairs of beacons whose distance is the same in
            // both sets of measurements.
            .beacon_offsets()
            .filter_map(|(offset, self_num)| {
                other_offsets
                    .get(&offset)
                    .map(|other_num| (self_num, *other_num))
            })
            // Generate a map from an index of self.beacons to a
            // vector of index pairs.
            .flat_map(|((s1, s2), other_indices)| {
                vec![s1, s2]
                    .into_iter()
                    .map(move |self_index| (self_index, other_indices))
            })
            .map(|(self_index, (o1, o2))| {
                (
                    self_index,
                    [o1, o2].iter().copied().collect::<HashSet<usize>>(),
                )
            })
            .into_group_map()
            .into_iter()
            // Take the intersection of each vector of index pairs.
            // This should uniquely determine the index of the
            // corresponding beacon in other.beacons.
            .flat_map(|(self_index, other_index_sets)| {
                other_index_sets
                    .into_iter()
                    .reduce(|acc, index_set| {
                        acc.intersection(&index_set).copied().collect()
                    })
                    .unwrap()
                    .into_iter()
                    .map(|other_index| (self_index, other_index))
                    .exactly_one()
                    .ok()
            })
            .collect();

        Ok(mapping)
    }

    fn locate_beacon(
        &self,
        other: &Self,
    ) -> Result<(Orientation, Displacement), Error> {
        let mapping = self.shared_beacon_mapping(other)?;

        if mapping.len() < 12 {
            return Err(Error::InsufficientSharedBeacons);
        }

        use DisorientedDisplacement as DD;

        let observation_pairs: Vec<(DD, DD)> = mapping
            .iter()
            .map(|(self_i, other_i)| {
                (self.beacons[*self_i], other.beacons[*other_i])
            })
            .collect();

        let offset_pairs: Vec<(Displacement, DD)> = observation_pairs
            .iter()
            .tuple_combinations()
            .map(|((self_a, other_a), (self_b, other_b))| {
                (*self_b - *self_a, *other_b - *other_a)
            })
            .map(|(self_diff, other_diff)| {
                (
                    self_diff.with_orientation(&Orientation::identity()),
                    other_diff,
                )
            })
            .collect();

        let orientation = Orientation::iter()
            .filter(|orientation| {
                offset_pairs.iter().all(|(self_offset, other_offset)| {
                    *self_offset == other_offset.with_orientation(orientation)
                })
            })
            .exactly_one()?;

        let location = observation_pairs
            .iter()
            .map(|(self_observed, other_observed)| {
                other_observed.with_orientation(&orientation)
                    - self_observed.with_orientation(&Orientation::identity())
            })
            .unique()
            .exactly_one()?;

        Ok((orientation, location))
    }

    fn merge_observations(&self, other: &Scanner) -> Result<Scanner, Error> {
        let (orientation, location) = self.locate_beacon(other)?;

        let beacons = other
            .beacons
            .iter()
            .map(|other_obs| {
                location + other_obs.with_orientation(&orientation)
            })
            // Mapping back to DisorientedDisplacement shouldn't be
            // necessary, probably means that I have too strict of a
            // type setup.
            .map(|val| DisorientedDisplacement(val.0))
            .chain(self.beacons.iter().copied())
            .unique()
            .collect();

        Ok(Self {
            id: self.id,
            beacons,
        })
    }
}

impl Day19 {
    fn parse_scanners(&self) -> Result<Vec<Scanner>, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(5))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let mut line_iter = puzzle_input.lines();
        let scanners =
            std::iter::from_fn(|| match Scanner::from_lines(&mut line_iter) {
                Ok(scanner) => Some(Ok(scanner)),
                Err(Error::UnexpectedEndOfStream) => None,
                Err(err) => Some(Err(err)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(scanners)
    }
}

impl Puzzle for Day19 {
    fn day(&self) -> i32 {
        19
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let scanners = self.parse_scanners()?;

        let mut queue: VecDeque<_> = scanners.into_iter().collect();
        let mut attempts_since_merge = 0;

        let mut state = queue.pop_front().unwrap();

        while queue.len() > 0 {
            let scanner = queue.pop_front().unwrap();

            let res = state.merge_observations(&scanner);

            match res {
                Ok(merged) => {
                    state = merged;
                    attempts_since_merge = 0;
                }
                Err(Error::InsufficientSharedBeacons) => {
                    queue.push_back(scanner);
                    attempts_since_merge += 1;
                    if attempts_since_merge > queue.len() {
                        return Err(Error::NeverFoundMatchedScanner);
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        let result = state.beacons.len();

        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let result = ();
        Ok(Box::new(result))
    }
}
