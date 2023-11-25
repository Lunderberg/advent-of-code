#![allow(unused_imports)]
use crate::{Error, Puzzle};

use std::convert::TryInto;
use std::marker::PhantomData;
use std::ops::Range;
use std::str::FromStr;

use itertools::Itertools;
use regex::Regex;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

#[derive(Debug)]
pub struct Command {
    new_state: bool,
    region: Cuboid,
}

#[derive(Debug, Clone)]
struct WorldState {
    enabled_regions: Vec<Cuboid>,
}

#[derive(Debug, Clone)]
struct Cuboid {
    ranges: [Range<i64>; 3],
}

impl WorldState {
    fn new() -> Self {
        Self {
            enabled_regions: Vec::new(),
        }
    }

    fn after_command(&self, command: &Command) -> Self {
        // Splitting out the untouched regions in necessary for
        // performance reasons, because Cuboid.difference can split
        // untouched regions.
        let (untouched, touched): (Vec<_>, Vec<_>) = self
            .enabled_regions
            .iter()
            .partition(|r| r.intersection(&command.region).is_empty());

        let enabled_regions: Vec<Cuboid> = touched
            .into_iter()
            .flat_map(|r| r.difference(&command.region))
            .chain(
                std::iter::repeat_with(|| command.region.clone())
                    .take(command.new_state as usize),
            )
            .chain(untouched.into_iter().cloned())
            .collect();

        Self { enabled_regions }
    }
}

impl Cuboid {
    fn size(&self) -> u64 {
        self.ranges
            .iter()
            .map(|r| {
                if r.is_empty() {
                    0
                } else {
                    (r.end - r.start) as u64
                }
            })
            .product()
    }

    // Return an iterator of cuboids whose union contains all points
    // that are in self, but not in other.
    fn difference<'a, 'b, 'c>(
        &'a self,
        other: &'b Self,
    ) -> impl Iterator<Item = Self> + 'c
    where
        'a: 'c,
        'b: 'c,
    {
        // I think this special case isn't needed, but let's see.
        // // Special case, there's no overlap present, so just return
        // // the original.
        // let entirely_distinct =
        //     self.ranges.iter().zip(other.ranges.iter()).any(
        //         |(self_range, other_range)| {
        //             (self_range.end < other_range.start)
        //                 || (other_range.end < self_range.end)
        //         },
        //     );
        // if entirely_distinct {
        //     return vec![self.clone()].into_iter();
        // }

        // Determine the allowed region subsets.

        self.ranges
            .iter()
            .zip(other.ranges.iter())
            // Map to boolean of (inside_self, inside_other, region).
            // Each list of ranges fills the region [a1,a2].
            .map(|(self_range, other_range)| {
                let a1 = self_range.start;
                let a2 = self_range.end;
                let b1 = other_range.start;
                let b2 = other_range.end;

                vec![a1, a2, b1, b2]
                    .into_iter()
                    .sorted()
                    .tuple_windows()
                    .filter(|(low, high)| low < high)
                    .filter(move |(low, _high)| self_range.contains(low))
                    .map(move |(low, high)| {
                        (other_range.contains(&low), low..high)
                    })
            })
            .multi_cartesian_product()
            // Filter out regions where all x/y/z coordinates overlap
            // with the other region (inside_other is true for all
            // dimensions).
            .filter_map(|subregion| {
                subregion
                    .iter()
                    .any(|(inside_other, _range)| !inside_other)
                    .then(|| {
                        let mut ranges = [0..0, 0..0, 0..0];

                        subregion
                            .into_iter()
                            .map(|(_inside_other, range)| range)
                            .zip_eq(ranges.iter_mut())
                            .for_each(|(range, out)| {
                                *out = range;
                            });
                        Cuboid { ranges }
                    })
            })
    }

    fn intersection(&self, other: &Self) -> Self {
        let mut ranges = [0..0, 0..0, 0..0];

        self.ranges
            .iter()
            .zip(other.ranges.iter())
            .map(|(self_range, other_range)| {
                let bottom = self_range.start.max(other_range.start);
                let top = self_range.end.min(other_range.end);
                bottom..top
            })
            .zip(ranges.iter_mut())
            .for_each(|(val, out)| {
                *out = val;
            });
        Self { ranges }
    }

    fn is_empty(&self) -> bool {
        self.ranges.iter().any(|r| r.is_empty())
    }
}

impl FromStr for Command {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let captures = Regex::new(
            r"(?x)
             (?P<state>(on)|(off))
             \s
             x=
             (?P<xmin>-?\d+)
             \.\.
             (?P<xmax>-?\d+)
             ,
             y=
             (?P<ymin>-?\d+)
             \.\.
             (?P<ymax>-?\d+)
             ,
             z=
             (?P<zmin>-?\d+)
             \.\.
             (?P<zmax>-?\d+)
             ",
        )
        .unwrap()
        .captures(s)
        .ok_or(Error::Mismatch)?;

        let new_state = captures.name("state").unwrap().as_str() == "on";

        let mut ranges = [0..0, 0..0, 0..0];
        ["xmin", "xmax", "ymin", "ymax", "zmin", "zmax"]
            .iter()
            .map(|name| captures.name(name).unwrap().as_str().parse::<i64>())
            .tuples()
            .map(|(a, b)| -> Result<_, Error> { Ok((a?, b?)) })
            // Problem gives inclusive range, but half-open ranges are
            // easier to work with.
            .map_ok(|(a, b)| a..(b + 1))
            .zip(ranges.iter_mut())
            .try_for_each(|(res, out)| -> Result<_, Error> {
                *out = res?;
                Ok(())
            })?;
        let region = Cuboid { ranges };

        Ok(Self { new_state, region })
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 2;

    type ParsedInput = Vec<Command>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let initialization_region = Cuboid {
            ranges: [-50..51, -50..51, -50..51],
        };

        let final_state =
            parsed.iter().fold(WorldState::new(), |state, command| {
                state.after_command(command)
            });

        Ok(final_state
            .enabled_regions
            .iter()
            .map(|region| region.intersection(&initialization_region).size())
            .sum::<u64>())
    }

    fn part_2(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let final_state =
            parsed.iter().fold(WorldState::new(), |state, command| {
                state.after_command(command)
            });

        Ok(final_state
            .enabled_regions
            .iter()
            .map(|region| region.size())
            .sum::<u64>())
    }
}
