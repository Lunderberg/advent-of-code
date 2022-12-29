#![allow(unused_imports)]
use crate::utils::geometry::Vector;
use crate::{Error, Puzzle};

use bit_set::BitSet;
use itertools::Itertools;

use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::ControlFlow;

#[derive(Debug, Clone)]
pub enum Jet {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RockShape {
    Horizontal,
    Plus,
    Angle,
    Vertical,
    Square,
}

#[derive(Debug, Clone)]
struct Rock {
    offset: Vector<2>,
    shape: RockShape,
}

#[derive(Debug)]
enum NextBoardStep {
    NewRock,
    PushRockWithJet(Rock),
    LetRockFall(Rock),
    SolidifyRock(Rock),
}

struct Board {
    next_step: NextBoardStep,
    filled: HashSet<Vector<2>>,
    width: u8,
}

#[derive(PartialEq, Eq, Hash)]
struct TopShape {
    filled: BitSet,
}

impl NextBoardStep {
    fn active_rock(&self) -> Option<&Rock> {
        match self {
            NextBoardStep::NewRock => None,
            NextBoardStep::PushRockWithJet(rock) => Some(rock),
            NextBoardStep::LetRockFall(rock) => Some(rock),
            NextBoardStep::SolidifyRock(rock) => Some(rock),
        }
    }
}

impl RockShape {
    fn iter_falling() -> impl Iterator<Item = RockShape> + Clone {
        use RockShape::*;
        vec![Horizontal, Plus, Angle, Vertical, Square].into_iter()
    }

    // Relative to the bottom left edge of the rock
    fn iter_shape(&self) -> impl Iterator<Item = Vector<2>> {
        match self {
            RockShape::Horizontal => {
                vec![[0, 0].into(), [1, 0].into(), [2, 0].into(), [3, 0].into()]
            }
            RockShape::Plus => vec![
                [0, 1].into(),
                [1, 0].into(),
                [1, 1].into(),
                [1, 2].into(),
                [2, 1].into(),
            ],
            RockShape::Angle => vec![
                [0, 0].into(),
                [1, 0].into(),
                [2, 0].into(),
                [2, 1].into(),
                [2, 2].into(),
            ],
            RockShape::Vertical => {
                vec![[0, 0].into(), [0, 1].into(), [0, 2].into(), [0, 3].into()]
            }
            RockShape::Square => {
                vec![[0, 0].into(), [0, 1].into(), [1, 0].into(), [1, 1].into()]
            }
        }
        .into_iter()
    }
}

impl Rock {
    fn iter_loc(&self) -> impl Iterator<Item = Vector<2>> + '_ {
        self.shape.iter_shape().map(move |p| p + self.offset)
    }

    fn after_move(&self, delta: Vector<2>) -> Self {
        Self {
            shape: self.shape,
            offset: self.offset + delta,
        }
    }
}

impl Jet {
    fn direction(&self) -> Vector<2> {
        match self {
            Jet::Left => [-1, 0].into(),
            Jet::Right => [1, 0].into(),
        }
    }
}

impl Board {
    fn run_to_completion(
        &mut self,
        shape_iter: &mut impl Iterator<Item = RockShape>,
        jet_iter: &mut impl Iterator<Item = Jet>,
    ) -> ControlFlow<()> {
        loop {
            self.advance(shape_iter, jet_iter)?;
        }
    }

    fn run_single_rock(
        &mut self,
        shape: RockShape,
        jet_iter: &mut impl Iterator<Item = Jet>,
    ) -> ControlFlow<()> {
        assert!(matches!(self.next_step, NextBoardStep::NewRock));
        self.run_to_completion(&mut std::iter::once(shape), jet_iter)
    }

    fn max_height(&self) -> i64 {
        self.filled.iter().map(|p| p.y()).max().unwrap_or(0)
    }

    // Flood-fill from the top.
    fn top_shape(&self) -> (TopShape, i64) {
        let max_y = self.max_height();

        let mut flood_fill: BitSet = BitSet::new();

        let mut to_visit: Vec<Vector<2>> = (1..self.width + 2)
            .map(|x| [x as i64, max_y + 1].into())
            .collect();
        let mut seen: HashSet<Vector<2>> = to_visit.iter().cloned().collect();
        let offsets: Vec<Vector<2>> =
            vec![[0, -1].into(), [1, 0].into(), [-1, 0].into()];

        while let Some(visiting) = to_visit.pop() {
            // println!("Visiting {visiting}, num remaining = {}", to_visit.len());
            offsets
                .iter()
                .cloned()
                .map(|offset| visiting + offset)
                .filter(|loc| {
                    let seen_before = seen.contains(loc);
                    if !seen_before {
                        seen.insert(*loc);
                    }
                    !seen_before
                })
                // .inspect(|loc| println!("Can visit {loc} from {visiting}"))
                .filter(|loc| self.is_in_bounds(*loc))
                .for_each(|loc| {
                    if self.is_open(loc) {
                        to_visit.push(loc);
                    } else {
                        let x_rel = (loc.x() - 1) as usize;
                        let y_rel = (max_y - loc.y()) as usize;
                        // println!("(xrel,yrel) = ({x_rel}, {y_rel})");
                        let linearized: usize =
                            y_rel * (self.width as usize) + x_rel;
                        flood_fill.insert(linearized);
                    }
                });
        }
        (TopShape { filled: flood_fill }, max_y)
    }

    fn advance(
        &mut self,
        shape_iter: &mut impl Iterator<Item = RockShape>,
        jet_iter: &mut impl Iterator<Item = Jet>,
    ) -> ControlFlow<()> {
        use NextBoardStep::*;

        let attempting_new_rock = matches!(self.next_step, NewRock);

        // println!("Step: {:?}", self.next_step);

        self.next_step = match std::mem::replace(&mut self.next_step, NewRock) {
            NewRock => {
                if let Some(shape) = shape_iter.next() {
                    let new_rock = Rock {
                        shape,
                        offset: self.new_rock_offset(),
                    };

                    PushRockWithJet(new_rock)
                } else {
                    NewRock
                }
            }
            PushRockWithJet(rock) => {
                let jet = jet_iter.next().expect("Reached end of jet iter");
                let new_rock =
                    self.try_move(&rock, jet.direction()).unwrap_or(rock);
                LetRockFall(new_rock)
            }
            LetRockFall(rock) => {
                if let Some(new_rock) = self.try_move(&rock, [0, -1].into()) {
                    PushRockWithJet(new_rock)
                } else {
                    SolidifyRock(rock)
                }
            }
            SolidifyRock(rock) => {
                self.solidify_rock(rock);
                NewRock
            }
        };

        // if attempting_new_rock {
        //     println!("{self}");
        //     pause();
        // }

        if attempting_new_rock && matches!(self.next_step, NewRock) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn new_rock_offset(&self) -> Vector<2> {
        let max_y = self.filled.iter().map(|p| p.y()).max().unwrap_or(0);
        [3, max_y + 4].into()
    }

    fn solidify_rock(&mut self, rock: Rock) {
        rock.iter_loc().for_each(|p| {
            self.filled.insert(p);
        });
    }

    fn is_in_bounds(&self, pos: Vector<2>) -> bool {
        pos.y() > 0 && pos.x() > 0 && pos.x() < (self.width as i64) + 1
    }

    fn is_open(&self, pos: Vector<2>) -> bool {
        self.is_in_bounds(pos) && !self.filled.contains(&pos)
    }

    fn try_move(&self, rock: &Rock, delta: Vector<2>) -> Option<Rock> {
        let new_rock = rock.after_move(delta);
        let can_move = new_rock.iter_loc().all(|s| self.is_open(s));
        if can_move {
            Some(new_rock)
        } else {
            None
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self {
            next_step: NextBoardStep::NewRock,
            filled: HashSet::new(),
            width: 7,
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let active_rock: HashSet<Vector<2>> = self
            .next_step
            .active_rock()
            .into_iter()
            .flat_map(|rock: &Rock| rock.iter_loc())
            .collect();

        let max_y: i64 = self
            .filled
            .iter()
            .chain(active_rock.iter())
            .map(|p| p.y())
            .max()
            .unwrap_or(0)
            .max(10);
        let min_y: i64 = (max_y - 20).max(-1);

        (min_y..=max_y).rev().try_for_each(|y| {
            let line = (0..(self.width as i64 + 2))
                .map(|x| {
                    let p = Vector::new([x, y]);

                    let is_side_wall = x == 0 || x == (self.width as i64) + 1;

                    if is_side_wall && y == 0 {
                        '+'
                    } else if is_side_wall {
                        '|'
                    } else if y == 0 {
                        '-'
                    } else if self.filled.contains(&p) {
                        '#'
                    } else if active_rock.contains(&p) {
                        '@'
                    } else {
                        '.'
                    }
                })
                .join("");

            let info = if y == min_y || y == max_y {
                format!("y = {y}")
            } else {
                "".to_string()
            };

            writeln!(f, "{line} {info}")
        })
    }
}

pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 1;
    const YEAR: u32 = 2022;
    const DAY: u8 = 17;

    type ParsedInput = Vec<Jet>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines
            .exactly_one()?
            .chars()
            .map(|c| match c {
                '<' => Ok(Jet::Left),
                '>' => Ok(Jet::Right),
                _ => Err(Error::UnknownChar(c)),
            })
            .collect()
    }

    type Part1Result = i64;
    fn part_1(jets: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        let mut board = Board::default();
        println!("Initial Board:\n{board}");

        board.run_to_completion(
            &mut RockShape::iter_falling().cycle().take(2022),
            &mut jets.iter().cloned().cycle(),
        );
        println!("Final Board:\n{board}");

        Ok(board.max_height())
    }

    type Part2Result = i64;
    fn part_2(jets: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        let iterations = 1000000000000usize;

        let mut iter_jet = jets.iter().cloned().enumerate().cycle().peekable();

        let cycle = RockShape::iter_falling().enumerate().cycle()
            .scan(Board::default(), |board, (rock_i,rock_shape)| {
                let jet_i: usize = iter_jet.peek().unwrap().0;
                board.run_single_rock(rock_shape, &mut iter_jet.by_ref().map(|(_i,jet)| jet));
                let (top_shape, height) = board.top_shape();
                Some(((rock_i,jet_i,top_shape), height))
            })
            .enumerate()
            .scan(
                HashMap::new(),
                |state: &mut HashMap<(usize,usize, TopShape), (usize,i64)>,
                 (i, (key,height))|
                  -> Option<((usize,i64), Option<(usize,i64)>)> {
                     let prev = state.get(&key).cloned();
                     let memo = (i,height);
                     state.insert(key, memo);
                     Some((memo,prev))
                },
            )
            .find_map(|((current_iter, current_height), opt_prev)| {
                opt_prev.map(|(prev_iter, prev_height)| {
                    (current_iter, current_height, prev_iter, prev_height)
                })
            })
            .expect("No cycle found");

        let (current_iter, current_height, prev_iter, prev_height) = cycle;
        let period: usize = current_iter - prev_iter;
        let height_from_skipped = (current_height - prev_height)
            * (((iterations - prev_iter) / period) as i64);
        let equivalent = prev_iter + (iterations - prev_iter) % period;

        let mut board = Board::default();
        board.run_to_completion(
            &mut RockShape::iter_falling().cycle().take(equivalent),
            &mut jets.iter().cloned().cycle(),
        );

        Ok(board.max_height() + height_from_skipped)
    }
}
