#![allow(unused_imports)]
use crate::utils::graph::DynamicGraph;
use crate::{Error, Puzzle};

use std::collections::{HashMap, VecDeque};
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::ops;
use std::str::FromStr;

use itertools::Itertools;

pub struct Day24;

#[derive(Debug)]
pub struct Program {
    instructions: Vec<Instruction>,
}

#[derive(Debug)]
struct ChecksumRound {
    x_add: i64,
    y_add: i64,
    z_div: i64,
}

#[derive(Debug)]
struct ChecksumGraph {
    rounds: Vec<ChecksumRound>,
    reverse_search: bool,
}

#[derive(Debug, Eq)]
struct ChecksumState {
    rounds_completed: usize,
    most_recent_digit: i64,
    z_value: i64,
}

impl std::cmp::PartialEq for ChecksumState {
    fn eq(&self, rhs: &Self) -> bool {
        (self.rounds_completed, self.z_value)
            == (rhs.rounds_completed, rhs.z_value)
    }
}

impl std::hash::Hash for ChecksumState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.rounds_completed.hash(state);
        self.most_recent_digit.hash(state);
        self.z_value.hash(state);
    }
}

impl ChecksumGraph {
    fn start(&self) -> ChecksumState {
        ChecksumState {
            rounds_completed: 0,
            most_recent_digit: 0,
            z_value: 0,
        }
    }
    fn end(&self) -> ChecksumState {
        ChecksumState {
            rounds_completed: self.rounds.len(),
            most_recent_digit: 0,
            z_value: 0,
        }
    }
    fn cost_of_digit(&self, used_in_round: usize, digit: i64) -> u64 {
        let rounds_remaining = self.rounds.len() - used_in_round - 1;
        let digit_cost = if self.reverse_search {
            digit
        } else {
            10 - digit
        };
        let cost = digit_cost * 10i64.pow(rounds_remaining as u32);
        cost as u64
    }

    fn find_serial(&self) -> Result<i64, Error> {
        Ok(self
            .shortest_path(self.start(), self.end())?
            .into_iter()
            .map(|(checksum_state, _)| checksum_state.most_recent_digit)
            .fold(0, |acc, digit| 10 * acc + digit))
    }
}

impl DynamicGraph<ChecksumState> for ChecksumGraph {
    fn connections_from(
        &self,
        node: &ChecksumState,
    ) -> Vec<(ChecksumState, u64)> {
        self.rounds
            .get(node.rounds_completed)
            .map(|round| {
                (1..=9)
                    .map(|digit| {
                        let z_value =
                            if digit == (node.z_value % 26 + round.x_add) {
                                node.z_value / round.z_div
                            } else {
                                26 * (node.z_value / round.z_div)
                                    + digit
                                    + round.y_add
                            };
                        let cost: u64 =
                            self.cost_of_digit(node.rounds_completed, digit);
                        (
                            ChecksumState {
                                rounds_completed: node.rounds_completed + 1,
                                most_recent_digit: digit,
                                z_value,
                            },
                            cost,
                        )
                    })
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }

    fn heuristic_between(
        &self,
        node_from: &ChecksumState,
        node_to: &ChecksumState,
    ) -> Option<u64> {
        let max_divisor = (node_from.rounds_completed
            ..node_to.rounds_completed)
            .map(|i| self.rounds[i].z_div)
            .product::<i64>();
        let max_value = (node_to.z_value + 1) * max_divisor - 1;

        if node_from == node_to {
            Some(0)
        } else if (node_from.rounds_completed >= node_to.rounds_completed)
            || (node_from.z_value > max_value)
        {
            None
        } else {
            let cheapest_digit = if self.reverse_search { 1 } else { 9 };
            Some(
                (node_from.rounds_completed..node_to.rounds_completed)
                    .map(|round_num| {
                        self.cost_of_digit(round_num, cheapest_digit)
                    })
                    .sum(),
            )
        }
    }
}

#[derive(Debug)]
enum Instruction {
    Input(MemoryLocation),
    Add(MemoryLocation, Argument),
    Mul(MemoryLocation, Argument),
    Div(MemoryLocation, Argument),
    Mod(MemoryLocation, Argument),
    Equal(MemoryLocation, Argument),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MemoryLocation {
    W,
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy)]
enum Argument {
    MemLoc(MemoryLocation),
    Int(i64),
}

#[derive(Debug)]
struct RuntimeState {
    vals: [i64; 4],
}

impl TryFrom<usize> for MemoryLocation {
    type Error = Error;
    fn try_from(i: usize) -> Result<Self, Error> {
        use MemoryLocation::*;
        match i {
            0 => Ok(W),
            1 => Ok(X),
            2 => Ok(Y),
            3 => Ok(Z),
            _ => Err(Error::InvalidIndex(i)),
        }
    }
}

impl MemoryLocation {
    fn index(&self) -> usize {
        use MemoryLocation::*;
        match self {
            W => 0,
            X => 1,
            Y => 2,
            Z => 3,
        }
    }
}

impl RuntimeState {
    fn _new() -> Self {
        Self { vals: [0; 4] }
    }

    fn _get_arg(&self, arg: &Argument) -> i64 {
        match arg {
            Argument::MemLoc(loc) => self[*loc],
            Argument::Int(num) => *num,
        }
    }
}

impl ops::Index<MemoryLocation> for RuntimeState {
    type Output = i64;
    fn index(&self, var: MemoryLocation) -> &i64 {
        &self.vals[var.index()]
    }
}

impl ops::IndexMut<MemoryLocation> for RuntimeState {
    fn index_mut(&mut self, var: MemoryLocation) -> &mut i64 {
        &mut self.vals[var.index()]
    }
}
impl Program {
    fn _execute<I>(&self, inputs: I) -> Result<RuntimeState, Error>
    where
        I: IntoIterator<Item = i64>,
    {
        let mut inputs = inputs.into_iter();

        self.instructions.iter().fold(
            Ok(RuntimeState::_new()),
            |res_state, inst| {
                use Instruction::*;

                res_state.and_then(|mut state| {
                    match inst {
                        Input(var) => {
                            state[*var] = inputs
                                .next()
                                .ok_or(Error::InsufficientInputValues)?;
                        }
                        Add(var, arg) => {
                            state[*var] += state._get_arg(arg);
                        }
                        Mul(var, arg) => {
                            state[*var] *= state._get_arg(arg);
                        }
                        Div(var, arg) => {
                            state[*var] /= state._get_arg(arg);
                        }
                        Mod(var, arg) => {
                            state[*var] %= state._get_arg(arg);
                        }
                        Equal(var, arg) => {
                            state[*var] =
                                (state[*var] == state._get_arg(arg)) as i64;
                        }
                    }
                    Ok(state)
                })
            },
        )
    }

    fn to_checksum_graph(
        &self,
        reverse_search: bool,
    ) -> Result<ChecksumGraph, Error> {
        // From inspection, the result of the MONAD is

        // def apply_value(serial):
        //     inputs = [int(c) for c in str(serial)]
        //
        //     # Each instruction matching "add x #"
        //     x_add = [10, 11, 14, 13, -6, -14, 14, 13, -8, -15, 10, -11, -13, -4]
        //     # Every 3rd instruction matching "add y #"
        //     y_add = [1, 9, 12, 6, 9, 15, 7, 12, 15, 3, 6, 2, 10, 12]
        //     # Every instruction matching "div z #"
        //     z_div = [1, 1, 1, 1, 26, 26, 1, 1, 26, 26, 1, 26, 26, 26]
        //
        //     z = 0
        //     for (w, dx, dy, z_div) in zip(inputs, x_add, y_add, z_div):
        //         z = z // z_div
        //         if w != z % 26 + dx:
        //             z = 26 * z + w + dy
        //
        //     return z

        let round_size = self
            .instructions
            .iter()
            .enumerate()
            .filter_map(|(i, inst)| match inst {
                Instruction::Input(_) => Some(i),
                _ => None,
            })
            .tuple_windows()
            .map(|(a, b)| b - a)
            .unique()
            .exactly_one()?;
        let rounds: Vec<_> = self
            .instructions
            .chunks(round_size)
            .map(|slice| -> Result<_, Error> {
                let x_add = slice
                    .iter()
                    .filter_map(|inst| match inst {
                        Instruction::Add(
                            MemoryLocation::X,
                            Argument::Int(dx),
                        ) => Some(dx),
                        _ => None,
                    })
                    .copied()
                    .exactly_one()?;
                let y_add = slice
                    .iter()
                    .filter_map(|inst| match inst {
                        Instruction::Add(
                            MemoryLocation::Y,
                            Argument::Int(dy),
                        ) => Some(dy),
                        _ => None,
                    })
                    .tuples()
                    .map(|(_, _, dy)| dy)
                    .copied()
                    .exactly_one()?;
                let z_div = slice
                    .iter()
                    .filter_map(|inst| match inst {
                        Instruction::Div(
                            MemoryLocation::Z,
                            Argument::Int(denom),
                        ) => Some(denom),
                        _ => None,
                    })
                    .copied()
                    .exactly_one()?;
                Ok(ChecksumRound {
                    x_add,
                    y_add,
                    z_div,
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(ChecksumGraph {
            rounds,
            reverse_search,
        })
    }
}

impl Display for MemoryLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let c = match self {
            MemoryLocation::W => 'w',
            MemoryLocation::X => 'x',
            MemoryLocation::Y => 'y',
            MemoryLocation::Z => 'z',
        };
        write!(f, "{}", c)
    }
}

impl Display for Argument {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Argument::MemLoc(var) => write!(f, "{}", var),
            Argument::Int(num) => write!(f, "{}", num),
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Instruction::*;
        match self {
            Input(var) => write!(f, "inp {}", var),
            Add(var, arg) => write!(f, "add {} {}", var, arg),
            Mul(var, arg) => write!(f, "mul {} {}", var, arg),
            Div(var, arg) => write!(f, "div {} {}", var, arg),
            Mod(var, arg) => write!(f, "mod {} {}", var, arg),
            Equal(var, arg) => write!(f, "eql {} {}", var, arg),
        }
    }
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.instructions
            .iter()
            .enumerate()
            .try_for_each(|(i, inst)| {
                let end = if (i + 1) == self.instructions.len() {
                    ""
                } else {
                    "\n"
                };
                write!(f, "{}{}", inst, end)
            })
    }
}

impl FromStr for Instruction {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        let mut iter = s.split(' ');

        let instruction = iter.next().ok_or(Error::UnexpectedEndOfStream)?;
        let var = iter
            .next()
            .ok_or(Error::UnexpectedEndOfStream)
            .and_then(|word| word.parse());
        let arg = iter
            .next()
            .ok_or(Error::UnexpectedEndOfStream)
            .and_then(|word| word.parse());

        match instruction {
            "inp" => Ok(Instruction::Input(var?)),
            "add" => Ok(Instruction::Add(var?, arg?)),
            "mul" => Ok(Instruction::Mul(var?, arg?)),
            "div" => Ok(Instruction::Div(var?, arg?)),
            "mod" => Ok(Instruction::Mod(var?, arg?)),
            "eql" => Ok(Instruction::Equal(var?, arg?)),
            _ => Err(Error::UnexpectedToken(instruction.to_string())),
        }
    }
}

impl FromStr for Argument {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        s.parse::<MemoryLocation>()
            .map(Argument::MemLoc)
            .or_else(|_| {
                s.parse::<i64>()
                    .map(Argument::Int)
                    .map_err(|_| Error::InvalidString(s.to_string()))
            })
    }
}

impl FromStr for MemoryLocation {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Error> {
        s.chars()
            .exactly_one()
            .map_err(|_| Error::InvalidString(s.to_string()))
            .and_then(|c| match c {
                'w' => Ok(MemoryLocation::W),
                'x' => Ok(MemoryLocation::X),
                'y' => Ok(MemoryLocation::Y),
                'z' => Ok(MemoryLocation::Z),
                _ => Err(Error::UnknownChar(c)),
            })
    }
}

impl Puzzle for Day24 {
    const YEAR: u32 = 2021;
    const DAY: u8 = 24;
    const IMPLEMENTED: bool = true;
    const EXAMPLE_NUM: u8 = 2;

    type ParsedInput = Program;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let instructions =
            lines.map(|line| line.parse()).collect::<Result<_, _>>()?;
        Ok(Program { instructions })
    }

    type Part1Result = i64;
    fn part_1(parsed: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        parsed.to_checksum_graph(false)?.find_serial()
    }

    type Part2Result = i64;
    fn part_2(parsed: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        parsed.to_checksum_graph(true)?.find_serial()
    }
}
