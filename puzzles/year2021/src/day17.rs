#![allow(unused_imports)]
use crate::utils::extensions::*;
use crate::{Error, Puzzle};

use core::ops::RangeInclusive;
use std::str::FromStr;

use itertools::Itertools;
use regex::Regex;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

#[derive(Debug)]
pub struct Target {
    x: RangeInclusive<i64>,
    y: RangeInclusive<i64>,
}

#[derive(Debug, Clone)]
struct Probe {
    xpos: i64,
    ypos: i64,
    xvel: i64,
    yvel: i64,
}

impl Target {
    fn initial_probes(&self) -> impl Iterator<Item = Probe> + '_ {
        self.yvel_range()
            .cartesian_product(self.xvel_range())
            .map(|(yvel, xvel)| Probe::new(xvel, yvel))
            .filter(move |probe| probe.path_intersects(self))
    }

    fn contains(&self, x: &i64, y: &i64) -> bool {
        self.x.contains(x) && self.y.contains(y)
    }

    fn xvel_range(&self) -> RangeInclusive<i64> {
        // x = sum(i, i=xvel..0)
        //   = sum(i, i=0..xvel)
        //   = xvel*(xvel+1)/2
        //   = xvel^2/2 + xvel/2
        // 0 = xvel^2/2 + xvel/2 - x
        // xvel = -(1/2) +/- sqrt(1/4 + 2x)

        // Smallest velocity that will eventually reach xmin.
        let xvel_min =
            ((((2 * self.x.start()) as f64) + 0.25).sqrt() - 0.5).ceil() as i64;

        // Largest velocity that wouldn't immediately overshoot xmax
        let xvel_max = *self.x.end();

        xvel_min..=xvel_max
    }

    fn yvel_range(&self) -> RangeInclusive<i64> {
        // Any lower and the probe would overshoot the target on the
        // first step.
        let yvel_min = self.bottom();

        // Any higher and the probe wouldn't reach its apex by the
        // time it has passed the x region.
        let yvel_max = *self.x.end();

        yvel_min..=yvel_max
    }

    fn bottom(&self) -> i64 {
        *self.y.start()
    }
}

impl Probe {
    fn new(xvel: i64, yvel: i64) -> Self {
        Self {
            xpos: 0,
            ypos: 0,
            xvel,
            yvel,
        }
    }

    fn path_intersects(&self, target: &Target) -> bool {
        self.path()
            .take_while(|state| state.could_intersect(target))
            .any(|state| target.contains(&state.xpos, &state.ypos))
    }

    fn path(&self) -> impl Iterator<Item = Probe> {
        std::iter::successors(Some(self.clone()), |state| Some(state.next()))
    }

    fn next(&self) -> Self {
        Self {
            xpos: self.xpos + self.xvel,
            ypos: self.ypos + self.yvel,
            xvel: (self.xvel.abs() - 1).max(0) * self.xvel.signum(),
            yvel: self.yvel - 1,
        }
    }

    fn ymax(&self) -> i64 {
        if self.yvel > 0 {
            let dist_to_apex = self.yvel * (self.yvel + 1) / 2;
            self.ypos + dist_to_apex
        } else {
            self.ypos
        }
    }

    fn x_range(&self) -> RangeInclusive<i64> {
        let x_dist_remaining = self.xvel * (self.xvel + 1) / 2;
        self.xpos..=(self.xpos + x_dist_remaining)
    }

    fn could_intersect(&self, target: &Target) -> bool {
        let could_reach_height = self.ymax() >= target.bottom();
        let could_reach_x = self.x_range().intersects(&target.x);

        could_reach_height && could_reach_x
    }
}

impl FromStr for Target {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        Regex::new(
            r"(?x)^target\sarea:\s
              x=
              (?P<xmin>-?\d+)
              \.\.
              (?P<xmax>-?\d+)
              ,\sy=
              (?P<ymin>-?\d+)
              \.\.
              (?P<ymax>-?\d+)
              $",
        )
        .unwrap()
        .captures(s)
        .map(|cap| {
            let (xmin, xmax, ymin, ymax) = ["xmin", "xmax", "ymin", "ymax"]
                .iter()
                .map(|name| {
                    cap.name(name).unwrap().as_str().parse::<i64>().unwrap()
                })
                .collect_tuple()
                .unwrap();
            Target {
                x: xmin..=xmax,
                y: ymin..=ymax,
            }
        })
        .ok_or_else(|| Error::InvalidString(s.to_string()))
    }
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Target;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.exactly_one()?.parse()
    }

    fn part_1(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(parsed
            .initial_probes()
            .map(|probe| probe.ymax())
            .max()
            .unwrap())
    }

    fn part_2(
        parsed: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(parsed.initial_probes().count())
    }
}
