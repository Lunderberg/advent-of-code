use std::{collections::VecDeque, fmt::Display, str::FromStr};

use aoc_utils::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Brick {
    name: Option<char>,
    start: Vector<3, i64>,
    end: Vector<3, i64>,
}

struct BrickSystem {
    bricks: Vec<Brick>,
}

impl FromStr for Brick {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (start, end) = s
            .split('~')
            .map(|pos_str| -> Result<_, Error> {
                pos_str
                    .split(',')
                    .map(|dim| dim.parse())
                    .collect::<Result<Vec<_>, _>>()?
                    .try_into()
                    .map(|pos: [i64; 3]| pos.into())
                    .map_err(|_| Error::WrongIteratorSize)
            })
            .collect_tuple()
            .ok_or(Error::WrongIteratorSize)?;

        Ok(Brick {
            name: None,
            start: start?,
            end: end?,
        })
    }
}
impl Display for Brick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} <==> {}",
            self.name.unwrap_or('?'),
            self.start,
            self.end
        )
    }
}

impl Brick {
    fn num_bricks(&self) -> usize {
        let dist = (self.end - self.start)
            .into_iter()
            .map(|delta| delta.abs() as usize)
            .sum::<usize>();
        dist + 1
    }

    fn iter_bricks(&self) -> impl Iterator<Item = Vector<3, i64>> + '_ {
        self.start.cardinal_points_to(&self.end)
    }

    fn is_above(&self, other: &Brick) -> bool {
        self.iter_bricks()
            .flat_map(|a| other.iter_bricks().map(move |b| (a, b)))
            .any(|(a, b)| a.x() == b.x() && a.y() == b.y() && a.z() > b.z())
    }

    fn is_supported_by(&self, other: &Brick) -> bool {
        other != self
            && self
                .iter_bricks()
                .flat_map(|a| other.iter_bricks().map(move |b| (a, b)))
                .any(|(a, b)| {
                    a.x() == b.x() && a.y() == b.y() && a.z() == b.z() + 1
                })
    }

    fn after_falling(&self, earlier_bricks: &Vec<Brick>) -> Self {
        let dist_to_support = earlier_bricks
            .iter()
            .flat_map(|other| other.iter_bricks())
            .flat_map(|other_brick| {
                self.iter_bricks()
                    .map(move |self_brick| (self_brick, other_brick))
            })
            .filter(|(self_brick, other_brick)| {
                self_brick.x() == other_brick.x()
                    && self_brick.y() == other_brick.y()
            })
            .map(|(self_brick, other_brick)| self_brick.z() - other_brick.z())
            .min()
            .unwrap_or_else(|| self.start.z().min(self.end.z()));

        let delta: Vector<3, i64> = [0, 0, -(dist_to_support - 1)].into();
        Self {
            name: self.name,
            start: self.start + delta,
            end: self.end + delta,
        }
    }
}

impl BrickSystem {
    fn new(bricks: Vec<Brick>) -> Self {
        Self { bricks }
    }

    fn topological_sort(self) -> Self {
        let mut in_bricks: VecDeque<_> = self.bricks.into();
        let mut out_bricks = Vec::new();

        while let Some(brick) = in_bricks.pop_front() {
            if in_bricks.iter().any(|other| brick.is_above(other)) {
                in_bricks.push_back(brick);
            } else {
                out_bricks.push(brick);
            }
        }

        Self { bricks: out_bricks }
    }

    fn after_falling(self) -> Self {
        let mut bricks = Vec::new();
        for brick in self.bricks {
            bricks.push(brick.after_falling(&bricks));
        }

        Self { bricks }
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<Brick>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines
            .enumerate()
            .map(|(i, line)| {
                let mut brick: Brick = line.parse()?;
                brick.name = if i < 26 {
                    char::from_u32(65 + i as u32)
                } else if i < 52 {
                    char::from_u32(97 + (i - 26) as u32)
                } else {
                    None
                };
                Ok(brick)
            })
            .collect()
    }

    fn part_1(
        bricks: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let system = BrickSystem::new(bricks.clone());
        println!("Initial:\n{}\n", system.bricks.iter().join("\n"));
        let system = system.topological_sort();
        println!(
            "After topological sort:\n{}\n",
            system.bricks.iter().join("\n")
        );
        let system = system.after_falling();
        println!("After falling:\n{}\n", system.bricks.iter().join("\n"));

        let num_unsafe_to_disintegrate = system
            .bricks
            .iter()
            .filter_map(|brick| {
                system
                    .bricks
                    .iter()
                    .filter(|other| brick.is_supported_by(other))
                    .exactly_one()
                    .ok()
                    .cloned()
            })
            .unique()
            .inspect(|other| println!("{other} is unsafe to disintegrate"))
            .count();
        let num_safe_to_disintegrate =
            system.bricks.len() - num_unsafe_to_disintegrate;

        Ok(num_safe_to_disintegrate)
    }

    fn part_2(_: &Self::ParsedInput) -> Result<impl std::fmt::Debug, Error> {
        Err::<(), _>(Error::NotYetImplemented)
    }
}
