#![allow(unused_imports)]
use crate::utils::geometry::Vector;
use crate::{Error, Puzzle};

use itertools::Itertools;
use std::fmt::{Display, Formatter};
use std::ops::RangeInclusive;

#[derive(Debug)]
pub struct Sensor {
    loc: Vector<2>,
    beacon: Vector<2>,
}

#[derive(Debug, Clone)]
struct Region1D {
    ranges: Vec<Option<RangeInclusive<i64>>>,
}

impl std::str::FromStr for Sensor {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.split(": ")
            .map(|s| s.strip_prefix("Sensor at ").unwrap_or(s))
            .map(|s| s.strip_prefix("closest beacon is at ").unwrap_or(s))
            .flat_map(|s| s.split(", "))
            .map(|s| s.strip_prefix("x=").unwrap_or(s))
            .map(|s| s.strip_prefix("y=").unwrap_or(s))
            .map(|s| s.parse::<i64>())
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .tuples()
            .map(|(a, b)| Vector::new([a, b]))
            .tuples()
            .map(|(loc, beacon)| Sensor { loc, beacon })
            .exactly_one()?)
    }
}

impl Display for Sensor {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Sensor at {}, connected to beacon at {}",
            self.loc, self.beacon
        )
    }
}

impl Sensor {
    fn range_covered(&self, row: i64) -> RangeInclusive<i64> {
        let dist = self.loc.manhattan_dist(&self.beacon);
        let dy = (self.loc.y() - row).abs();
        let max_dx = dist - dy;
        let xmin = self.loc.x() - max_dx;
        let xmax = self.loc.x() + max_dx;
        xmin..=xmax
    }
}

impl From<RangeInclusive<i64>> for Region1D {
    fn from(range: RangeInclusive<i64>) -> Self {
        Self {
            ranges: vec![Some(range)],
        }
    }
}

impl Region1D {
    fn total_elements(&self) -> i64 {
        self.ranges
            .iter()
            .filter_map(|range| range.as_ref())
            .map(|range| range.end() - range.start() + 1)
            .sum()
    }

    fn union(&self, other: &Region1D) -> Region1D {
        Region1D {
            ranges: self
                .ranges
                .iter()
                .chain(other.ranges.iter())
                .cloned()
                .collect(),
        }
        .simplify()
    }

    fn restrict(&self, window: RangeInclusive<i64>) -> Self {
        Region1D {
            ranges: self
                .ranges
                .iter()
                .flatten()
                .map(|range| {
                    let start: i64 =
                        *range.start().clamp(window.start(), window.end());
                    let end: i64 =
                        *range.end().clamp(window.start(), window.end());
                    Some(start..=end)
                })
                .collect(),
        }
        .simplify()
    }

    fn combinable(&self) -> Option<(usize, usize, RangeInclusive<i64>)> {
        self.ranges
            .iter()
            .enumerate()
            .filter_map(|(i, opt_range)| {
                opt_range.as_ref().map(|range| (i, range))
            })
            .tuple_combinations()
            .find(|((_, a), (_, b))| {
                a.contains(b.start()) || b.contains(a.start())
            })
            .map(|((i, a), (j, b))| {
                let min = *a.start().min(b.start());
                let max = *a.end().max(b.end());
                (i, j, min..=max)
            })
    }

    fn simplify(mut self) -> Self {
        while let Some((i, j, merged)) = self.combinable() {
            self.ranges[i] = Some(merged);
            self.ranges[j] = None;
        }
        let ranges = self
            .ranges
            .into_iter()
            .filter(|range| range.is_none())
            .collect();
        Self { ranges }
    }
}

pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;
    const YEAR: u32 = 2022;
    const DAY: u8 = 15;

    type ParsedInput = Vec<Sensor>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    type Part1Result = usize;
    fn part_1(sensors: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        // The target row is different for the example and the actual input.
        let row_y = if sensors.len() == 14 { 10 } else { 2000000 };

        let ranges: Vec<_> = sensors
            .iter()
            .map(|sensor| sensor.range_covered(row_y))
            .filter(|range| !range.is_empty())
            .collect();

        let beacons_on_row = sensors
            .iter()
            .map(|sensor| sensor.beacon)
            .filter(|loc| loc.y() == row_y)
            .unique()
            .count();

        let xmin = *ranges.iter().map(|range| range.start()).min().unwrap();
        let xmax = *ranges.iter().map(|range| range.end()).max().unwrap();

        let _region = sensors
            .iter()
            .map(|sensor| sensor.range_covered(row_y))
            .map(|range| -> Region1D { range.into() })
            .reduce(|a, b| a.union(&b));

        let tiles_observed = (xmin..=xmax)
            .filter(|i| ranges.iter().any(|range| range.contains(i)))
            .count();

        Ok(tiles_observed - beacons_on_row)
    }

    type Part2Result = i64;
    fn part_2(sensors: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        // The target row is different for the example and the actual input.
        let search_range = if sensors.len() == 14 {
            0..=20
        } else {
            0..=4000000
        };

        let row_elements = search_range.end() - search_range.start() + 1;

        let distress_beacon = (search_range.clone())
            .map(|y| {
                (
                    y,
                    sensors
                        .iter()
                        .map(|sensor| sensor.range_covered(y))
                        .filter(|range| !range.is_empty())
                        .map(|range| -> Region1D { range.into() })
                        .reduce(|a, b| a.union(&b))
                        .unwrap()
                        .restrict(search_range.clone()),
                )
            })
            .find(|(_y, region)| region.total_elements() < row_elements)
            .map(|(y, region)| -> Vector<2> {
                [
                    region
                        .ranges
                        .iter()
                        .filter_map(|opt_range| {
                            opt_range.as_ref().map(|range| range.end() + 1)
                        })
                        .min()
                        .unwrap(),
                    y,
                ]
                .into()
            })
            .ok_or(Error::NoneError)?;

        Ok(distress_beacon.x() * 4000000 + distress_beacon.y())
    }
}
