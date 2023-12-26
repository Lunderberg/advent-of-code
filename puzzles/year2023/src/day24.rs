use std::{collections::HashMap, fmt::Display, str::FromStr};

use aoc_utils::prelude::*;
use indicatif::ProgressIterator as _;

pub struct Storm {
    hail: Vec<Hail>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Hail {
    position: Vector<3, i128>,
    velocity: Vector<3, i128>,
}

impl FromStr for Hail {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let (position, velocity) = line
            .split('@')
            .flat_map(|s| s.split(','))
            .map(|s| s.trim().parse())
            .tuples()
            .map(|(x, y, z)| -> Result<_, Error> { Ok([x?, y?, z?].into()) })
            .collect_tuple()
            .ok_or(Error::WrongIteratorSize)?;

        Ok(Hail {
            position: position?,
            velocity: velocity?,
        })
    }
}
impl Display for Hail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{pos {}, vel {}}}", self.position, self.velocity)
    }
}
impl Display for Storm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.hail.iter().try_for_each(|hail| writeln!(f, "{hail}"))
    }
}

trait VectorExt {
    fn is_in_test_area(&self) -> bool;
}
impl VectorExt for Vector<3, i128> {
    fn is_in_test_area(&self) -> bool {
        let min = 200000000000000;
        let max = 400000000000000;
        min <= self.x() && self.x() <= max && min <= self.y() && self.y() <= max
    }
}

impl Storm {
    fn iter_pairs(&self) -> impl Iterator<Item = (Hail, Hail)> + '_ {
        self.hail.iter().cloned().tuple_combinations()
    }
}

impl Hail {
    fn xy_intersection(
        &self,
        other: &Hail,
    ) -> Option<Vector<2, Fraction<i128>>> {
        let v1: Vector<2, Fraction<i128>> =
            [self.velocity.x().into(), self.velocity.y().into()].into();
        let p1: Vector<2, Fraction<i128>> =
            [self.position.x().into(), self.position.y().into()].into();

        let v2: Vector<2, Fraction<i128>> =
            [other.velocity.x().into(), other.velocity.y().into()].into();
        let p2: Vector<2, Fraction<i128>> =
            [other.position.x().into(), other.position.y().into()].into();

        // For first hailstone,
        //
        // x = v1.x*t + p1.x
        // y = v1.y*t + p1.y
        //
        // v1.y*x = v1.y*v1.x*t + v1.y*p1.x
        // v1.x*y = v1.x*v1.y*t + v1.x*p1.y
        //
        // v1.y*x - v1.x*y = v1.y*p1.x - v1.x*p1.y

        // Analogously, for second hailstone,
        // v2.y*x - v2.x*y = v2.y*p2.x - v2.x*p2.y

        // ┌              ┐┌   ┐   ┌                       ┐
        // | v1.y   -v1.x || x | = | v1.y*p1.x - v1.x*p1.y |
        // | v2.y   -v2.x || y |   | v2.y*p2.x - v2.x*p2.y |
        // └              ┘└   ┘   └                       ┘
        //
        // ┌              ┐-1    ┌             ┐
        // | v1.y   -v1.x |    = | -v2.x  v1.x | / (v2.y*v1.x - v2.x*v1.y)
        // | v2.y   -v2.x |      | -v2.y  v1.y |
        // └              ┘      └             ┘
        // D = (v2.y*v1.x - v2.x*v1.y)
        //
        // ┌   ┐    ┌              ┐┌                       ┐
        // | x | =  | -v2.x  v1.x  || v1.y*p1.x - v1.x*p1.y | / D
        // | y |    | -v2.y  v1.y  || v2.y*p2.x - v2.x*p2.y |
        // └   ┘    └              ┘└                       ┘
        // ┌   ┐    ┌                                                              ┐
        // | x | =  | -v2.x*(v1.y*p1.x - v1.x*p1.y) + v1.x*(v2.y*p2.x - v2.x*p2.y) | / D
        // | y |    | -v2.y*(v1.y*p1.x - v1.x*p1.y) + v1.y*(v2.y*p2.x - v2.x*p2.y) |
        // └   ┘    └                                                              ┘

        let d = (v2.y() * v1.x() - v2.x() * v1.y()).normalize();

        if d == 0 {
            return None;
        }

        let a = (v1.y() * p1.x() - v1.x() * p1.y()).normalize();
        let b = (v2.y() * p2.x() - v2.x() * p2.y()).normalize();

        let x = (v1.x() * b - v2.x() * a) / d;
        let y = (v1.y() * b - v2.y() * a) / d;
        let pos: Vector<2, Fraction<i128>> = [x, y].into();

        let is_p1_future = (pos - p1)
            .into_iter()
            .zip(v1.into_iter())
            .all(|(delta, v)| delta * v.into() >= 0.into());
        let is_p2_future = (pos - p2)
            .into_iter()
            .zip(v2.into_iter())
            .all(|(delta, v)| delta * v.into() >= 0.into());

        if is_p1_future && is_p2_future {
            Some(pos)
        } else {
            None
        }
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Storm;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let hail = lines.map(|line| line.parse()).collect::<Result<_, _>>()?;
        Ok(Storm { hail })
    }

    fn part_1(
        storm: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let test_area: std::ops::RangeInclusive<Fraction<i128>> =
            if storm.hail.len() == 5 {
                7.into()..=27.into()
            } else {
                200000000000000.into()..=400000000000000.into()
            };

        let num_intersect = storm
            .iter_pairs()
            .filter_map(|(a, b)| a.xy_intersection(&b))
            .filter(|intersection| {
                test_area.contains(&intersection.x())
                    && test_area.contains(&intersection.y())
            })
            .count();
        Ok(num_intersect)
    }

    fn part_2(
        storm: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        println!("Num hail: {}", storm.hail.len());

        let pos_lookup: HashMap<_, _> = storm
            .hail
            .iter()
            .enumerate()
            .flat_map(|(i, hail)| {
                (0..=storm.hail.len())
                    .map(move |t| {
                        let t = t as i128;
                        hail.position + hail.velocity * t
                    })
                    .map(move |pos| (pos, i))
            })
            .collect();
        println!("Lookup size: {}", pos_lookup.len());

        // pi = p0 + v0*ti

        let best_rock = storm
            .iter_pairs()
            .flat_map(|(a, b)| [(a.clone(), b.clone()), (b, a)])
            // .progress_count((storm.hail.len() * (storm.hail.len() - 1)) as u64)
            .map(|(a, b)| {
                let p1 = a.position + a.velocity * 1;
                let p2 = b.position + b.velocity * 2;
                let v0 = p2 - p1;
                let p0 = p1 - v0;
                // println!("From {a} at t=1 to {b} at t=2");
                // println!("\tp0 = {p0}, v0 = {v0}");
                Hail {
                    position: p0,
                    velocity: v0,
                }
            })
            .max_by_key(|rock| {
                (0..=storm.hail.len())
                    .filter_map(|t| {
                        let t = t as i128;
                        let pos = rock.position + rock.velocity * t;
                        pos_lookup.get(&pos)
                    })
                    .unique()
                    .count()
            })
            .unwrap();

        let best_rock_collisions = (0..=storm.hail.len())
            .filter_map(|t| {
                let t = t as i128;
                let pos = best_rock.position + best_rock.velocity * t;
                pos_lookup.get(&pos)
            })
            .unique()
            .count();

        println!("Best rock: {best_rock:?}");
        println!("Num collisions: {best_rock_collisions}");

        Ok(best_rock.position.into_iter().sum::<i128>())
    }
}
