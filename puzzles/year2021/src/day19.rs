use aoc_utils::prelude::*;

use std::collections::{HashMap, HashSet, VecDeque};

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

#[derive(Debug, Clone)]
pub struct Scanner {
    beacons: Vec<Vector<3>>,
}

#[derive(Debug)]
struct ScannerSet {
    beacons: Vec<Vector<3>>,
    scanners: Vec<Vector<3>>,
}

impl Scanner {
    fn from_lines<'a>(
        lines: &mut impl Iterator<Item = &'a str>,
    ) -> Result<Self, Error> {
        let _id_line = lines.next().ok_or(Error::UnexpectedEndOfStream)?;

        let beacons = lines
            .by_ref()
            .take_while(|line| !line.is_empty())
            .map(|line| line.parse::<Vector<3>>())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { beacons })
    }
}

impl ScannerSet {
    fn new(scanner: &Scanner) -> Self {
        let beacons = scanner.beacons.clone();
        let scanners = vec![Vector::<3>::new([0, 0, 0])];
        Self { beacons, scanners }
    }

    // Key to uniquely identify pairs of beacons.  Each pair has
    // an offset in (x,y,z) space.  The assignment of coordinates
    // and the signs are uncertain, so the .abs() and sorting
    // produce a consistent key.
    fn relative_key(a: &Vector<3>, b: &Vector<3>) -> [i64; 3] {
        let mut output = [0; 3];

        (*a - *b)
            .iter()
            .map(|d| d.abs())
            .sorted()
            .zip(output.iter_mut())
            .for_each(|(val, out)| {
                *out = val;
            });

        output
    }

    // Generator of all combinations between observed beacons.
    fn beacon_offsets(
        beacons: &[Vector<3>],
    ) -> impl Iterator<Item = ([i64; 3], HashSet<usize>)> + '_ {
        beacons.iter().enumerate().tuple_combinations().map(
            |((ia, a), (ib, b))| {
                (Self::relative_key(b, a), [ia, ib].iter().copied().collect())
            },
        )
    }

    fn identify_shared_observations(
        observed_a: &[Vector<3>],
        observed_b: &[Vector<3>],
    ) -> Result<HashMap<usize, usize>, Error> {
        // Make a lookup map from the (a,b,c) offset array into the
        // pair of other.beacons indices that generates that offset.
        let other_offsets: HashMap<_, _> =
            Self::beacon_offsets(observed_b).collect();

        let mapping: HashMap<_, _> = Self::beacon_offsets(observed_a)
            // Identify pairs of beacons whose distance is the same in
            // both sets of measurements.
            .filter_map(|(offset, self_num)| {
                other_offsets
                    .get(&offset)
                    .map(|other_num| (self_num, other_num))
            })
            // Generate a map from an index of self.beacons to a
            // vector of index pairs.
            .flat_map(|(self_indices, other_indices)| {
                self_indices
                    .into_iter()
                    .map(move |self_index| (self_index, other_indices))
            })
            .into_group_map()
            .into_iter()
            // Take the intersection of each vector of index pairs.
            // This should uniquely determine the index of the
            // corresponding beacon in other.beacons.
            .flat_map(|(self_index, other_index_sets)| {
                other_index_sets
                    .into_iter()
                    .cloned()
                    .reduce(|acc, index_set| {
                        acc.intersection(&index_set).copied().collect()
                    })
                    .unwrap()
                    .into_iter()
                    .map(|other_index| (self_index, other_index))
                    .exactly_one_or_err()
                    .ok()
            })
            .collect();

        Ok(mapping)
    }

    fn locate_coordinate_system(
        self_beacons: &[Vector<3>],
        other_beacons: &[Vector<3>],
    ) -> Result<(Matrix<3, 3>, Vector<3>), Error> {
        let mapping = ScannerSet::identify_shared_observations(
            self_beacons,
            other_beacons,
        )?;

        if mapping.len() < 12 {
            return Err(Error::InsufficientSharedBeacons);
        }

        let observation_pairs: Vec<_> = mapping
            .iter()
            .map(|(self_i, other_i)| {
                (self_beacons[*self_i], other_beacons[*other_i])
            })
            .collect();

        let offset_pairs: Vec<_> = observation_pairs
            .iter()
            .tuple_combinations()
            .map(|((self_a, other_a), (self_b, other_b))| {
                (*self_b - *self_a, *other_b - *other_a)
            })
            .map(|(self_diff, other_diff)| (self_diff, other_diff))
            .collect();

        let rotation_matrix = Matrix::<3, 3>::iter_90degrees()
            .filter(|mat| {
                offset_pairs.iter().all(|(self_offset, other_offset)| {
                    *self_offset == *mat * *other_offset
                })
            })
            .exactly_one_or_err()?;

        let location = observation_pairs
            .iter()
            .map(|(self_observed, other_observed)| {
                *self_observed - rotation_matrix * *other_observed
            })
            .unique()
            .exactly_one_or_err()?;

        Ok((rotation_matrix, location))
    }

    fn merge_observations(&self, other: &Scanner) -> Result<Self, Error> {
        let (rotation_matrix, location) = ScannerSet::locate_coordinate_system(
            &self.beacons,
            &other.beacons,
        )?;

        let beacons = other
            .beacons
            .iter()
            .map(|other_obs| location + rotation_matrix * *other_obs)
            .chain(self.beacons.iter().copied())
            .unique()
            .collect();

        let scanners = self
            .scanners
            .iter()
            .copied()
            .chain(std::iter::once(location))
            .collect();

        Ok(Self { beacons, scanners })
    }

    fn merge_all<'a>(
        scanners: impl Iterator<Item = &'a Scanner>,
    ) -> Result<Self, Error> {
        let mut queue: VecDeque<_> = scanners.collect();
        let mut attempts_since_merge = 0;

        let mut state = Self::new(queue.pop_front().unwrap());

        while !queue.is_empty() {
            let scanner = queue.pop_front().unwrap();

            let res = state.merge_observations(scanner);

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

        Ok(state)
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 5;

    type ParsedInput = Vec<Scanner>;
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let scanners =
            std::iter::from_fn(|| match Scanner::from_lines(&mut lines) {
                Ok(scanner) => Some(Ok(scanner)),
                Err(Error::UnexpectedEndOfStream) => None,
                Err(err) => Some(Err(err)),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(scanners)
    }

    fn part_1(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let merged = ScannerSet::merge_all(parsed.iter())?;
        Ok(merged.beacons.len())
    }

    fn part_2(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let merged = ScannerSet::merge_all(parsed.iter())?;

        Ok(merged
            .scanners
            .into_iter()
            .tuple_combinations()
            .map(|(a, b)| a.manhattan_dist(&b))
            .max()
            .unwrap())
    }
}
