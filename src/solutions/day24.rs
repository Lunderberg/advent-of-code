#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use std::collections::{HashMap, VecDeque};
use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::ops;
use std::str::FromStr;

use itertools::Itertools;

pub struct Day24;

#[derive(Debug)]
struct Program {
    instructions: Vec<Instruction>,
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
    fn new() -> Self {
        Self { vals: [0; 4] }
    }

    fn get_arg(&self, arg: &Argument) -> i64 {
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
    fn execute<I>(&self, inputs: I) -> Result<RuntimeState, Error>
    where
        I: IntoIterator<Item = i64>,
    {
        let mut inputs = inputs.into_iter();

        self.instructions.iter().fold(
            Ok(RuntimeState::new()),
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
                            state[*var] = state[*var] + state.get_arg(arg);
                        }
                        Mul(var, arg) => {
                            state[*var] = state[*var] * state.get_arg(arg);
                        }
                        Div(var, arg) => {
                            state[*var] = state[*var] / state.get_arg(arg);
                        }
                        Mod(var, arg) => {
                            state[*var] = state[*var] % state.get_arg(arg);
                        }
                        Equal(var, arg) => {
                            state[*var] =
                                (state[*var] == state.get_arg(arg)) as i64;
                        }
                    }
                    Ok(state)
                })
            },
        )
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
            .map(|var| Argument::MemLoc(var))
            .or_else(|_| {
                s.parse::<i64>()
                    .map(|val| Argument::Int(val))
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

impl Day24 {
    fn parse_instructions(&self) -> Result<Program, Error> {
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(1))?;
        //let puzzle_input = self.puzzle_input(PuzzleInput::Example(2))?;
        let puzzle_input = self.puzzle_input(PuzzleInput::User)?;

        let instructions = puzzle_input
            .lines()
            .map(|line| line.parse())
            .collect::<Result<_, _>>()?;
        Ok(Program { instructions })
    }
}

impl Puzzle for Day24 {
    fn day(&self) -> i32 {
        24
    }
    fn implemented(&self) -> bool {
        true
    }
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let program = self.parse_instructions()?;

        println!("Program:\n{}", program);

        {
            let serial: i64 = 13579246899999;
            let num_digits = (serial as f64).log10().ceil() as u32;
            let digits =
                (0..num_digits).rev().map(|i| (serial / 10_i64.pow(i)) % 10);

            let _result = program.execute(digits);
        }

        let result = ();
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let result = ();
        Ok(Box::new(result))
    }
}
