#![allow(unused_imports)]
use crate::utils::Error;
use crate::utils::{Expression, Variable};
use crate::utils::{Puzzle, PuzzleExtensions, PuzzleInput};

use std::collections::{HashMap, HashSet, VecDeque};
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

#[derive(Debug, Clone, Copy)]
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
struct RuntimeState<T> {
    vals: [T; 4],
}

#[derive(Debug)]
struct ProgramValues {
    target_vars: Vec<Variable>,
    var_names: HashMap<Variable, String>,
    states: Vec<RuntimeState<Expression>>,
    instructions: Vec<Instruction>,
}

impl MemoryLocation {
    fn locations() -> [MemoryLocation; 4] {
        use MemoryLocation::*;
        [W, X, Y, Z]
    }

    fn iter() -> impl Iterator<Item = Self> {
        use MemoryLocation::*;
        vec![W, X, Y, Z].into_iter()
    }

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

impl Instruction {
    fn loc(&self) -> MemoryLocation {
        use Instruction::*;
        match self {
            Input(loc) => *loc,
            Add(loc, ..) => *loc,
            Mul(loc, ..) => *loc,
            Div(loc, ..) => *loc,
            Mod(loc, ..) => *loc,
            Equal(loc, ..) => *loc,
        }
    }

    fn arg(&self) -> Option<Argument> {
        use Instruction::*;
        match self {
            Input(..) => None,
            Add(_, arg) => Some(*arg),
            Mul(_, arg) => Some(*arg),
            Div(_, arg) => Some(*arg),
            Mod(_, arg) => Some(*arg),
            Equal(_, arg) => Some(*arg),
        }
    }
}

impl<T> RuntimeState<T>
where
    T: Clone,
{
    fn new<F>(mut builder: F) -> Self
    where
        F: FnMut() -> T,
    {
        let vals = [0; 4].map(|_| builder());
        Self { vals }
    }

    fn map<B, F>(&self, mut f: F) -> RuntimeState<B>
    where
        F: FnMut(&T) -> B,
    {
        let vals = [0, 1, 2, 3].map(|i| f(&self.vals[i]));
        RuntimeState::<B> { vals }
    }

    fn iter(&self) -> impl Iterator<Item = (MemoryLocation, &T)> + '_ {
        let locs: Vec<_> = MemoryLocation::locations().into();

        locs.into_iter().map(move |loc| (loc, &self[loc]))
    }
}

impl<T> RuntimeState<T>
where
    T: From<i64>,
    T: Clone,
{
    fn get_arg(&self, arg: &Argument) -> T {
        match arg {
            Argument::MemLoc(loc) => self[*loc].clone(),
            Argument::Int(num) => (*num).into(),
        }
    }
}

impl<T> ops::Index<MemoryLocation> for RuntimeState<T> {
    type Output = T;
    fn index(&self, var: MemoryLocation) -> &T {
        &self.vals[var.index()]
    }
}

impl<T> ops::IndexMut<MemoryLocation> for RuntimeState<T> {
    fn index_mut(&mut self, var: MemoryLocation) -> &mut T {
        &mut self.vals[var.index()]
    }
}

impl Program {
    fn execute<I>(&self, inputs: I) -> Result<RuntimeState<i64>, Error>
    where
        I: IntoIterator<Item = i64>,
    {
        let mut inputs = inputs.into_iter();

        self.instructions.iter().fold(
            Ok(RuntimeState::new(|| 0)),
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

impl ProgramValues {
    fn new(program: &Program) -> Self {
        let instructions = program.instructions.clone();
        let variables: Vec<RuntimeState<Variable>> = (0..=instructions.len())
            .map(|_| RuntimeState::new(Variable::new))
            .collect();
        let states: Vec<RuntimeState<Expression>> = variables
            .iter()
            .map(|vars| vars.map(|var| (*var).into()))
            .collect();
        let var_names = variables
            .iter()
            .enumerate()
            .flat_map(|(i, state)| {
                state.iter().map(move |(loc, var)| (i, loc, var))
            })
            .map(|(i, loc, var)| (*var, format!("{}{}", loc, i)))
            .collect();

        Self {
            states,
            instructions,
            var_names,
            target_vars: Vec::new(),
        }
    }

    fn apply_constraints(&mut self, constraints: &Vec<Expression>) {
        let mut known_vars: HashSet<Variable> =
            self.target_vars.iter().copied().collect();
        let mut known_exprs: HashMap<Variable, Expression> = HashMap::new();

        let mut to_check: VecDeque<Expression> =
            constraints.iter().cloned().collect();
        let mut checked_since_last_success = 0;

        while to_check.len() > 0 && checked_since_last_success < to_check.len()
        {
            let equality = to_check.pop_front().unwrap();

            let solved_var = equality
                .variables()
                .difference(&known_vars)
                .sorted()
                .rev()
                .find_map(|&var| {
                    equality.solve_for(var).map(move |expr| (var, expr))
                });

            if let Some((var, expr)) = solved_var {
                // Update the existing equalities and derived
                // expressions.

                to_check = to_check
                    .iter()
                    .map(|prev_expr| {
                        prev_expr.substitute(var, &expr).simplify()
                    })
                    .enumerate()
                    .map(|(_i, expr)| expr)
                    .collect();
                known_exprs = known_exprs
                    .into_iter()
                    .map(|(prev_var, prev_expr)| {
                        (prev_var, prev_expr.substitute(var, &expr).simplify())
                    })
                    .collect();

                // Mark this variable as known
                known_vars.insert(var);
                known_exprs.insert(var, expr);

                checked_since_last_success = 0;
            } else {
                // Push the equality back onto the queue, maybe it'll
                // be easier to solve next time around.
                to_check.push_back(equality);
                checked_since_last_success += 1;
            }
        }

        self.states = self
            .states
            .iter()
            .map(|state| {
                state.map(|expr| {
                    known_exprs
                        .iter()
                        .fold(expr.clone(), |acc, (known_var, known_expr)| {
                            acc.substitute(*known_var, known_expr)
                        })
                        .simplify()
                })
            })
            .collect();
    }

    fn forward_constraints(&self) -> Vec<Expression> {
        self.states
            .iter()
            .tuple_windows()
            .zip_eq(self.instructions.iter())
            .flat_map(|((before, after), inst)| {
                MemoryLocation::iter()
                    .filter(move |loc| inst.loc() != *loc)
                    .map(move |loc| (before[loc].clone(), after[loc].clone()))
            })
            .map(|(before, after)| after.equal_value(before))
            .collect()
    }

    // fn mark_inputs(&mut self) {

    // }

    // fn constraints(&self) -> impl Iterator<Item=(Expression,Expression)> + '_ {
    //     self.states.t
    // }
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

impl Display for ProgramValues {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let instruction_before = std::iter::once(None)
            .chain(self.instructions.iter().map(|inst| Some(inst)));
        self.states.iter().zip(instruction_before).try_for_each(
            |(state, inst)| {
                if let Some(inst) = inst {
                    write!(f, "{}\n", inst)?;
                } else {
                    write!(f, "Initial state\n")?;
                }

                MemoryLocation::locations().iter().try_for_each(|loc| {
                    write!(
                        f,
                        "\t{} = {}\n",
                        loc,
                        state[*loc].format(&self.var_names)
                    )
                })?;

                Ok(())
            },
        )
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

        //println!("Program:\n{}", program);

        {
            let serial: i64 = 13579246899999;
            let num_digits = (serial as f64).log10().ceil() as u32;
            let digits =
                (0..num_digits).rev().map(|i| (serial / 10_i64.pow(i)) % 10);

            let _result = program.execute(digits);
        }

        let mut flow = ProgramValues::new(&program);
        flow.apply_constraints(&flow.forward_constraints());
        println!("{}", flow);
        // flow.forward_constraints()
        //     .into_iter()
        //     .for_each(|c| println!("{}", c));

        let result = ();
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let result = ();
        Ok(Box::new(result))
    }
}
