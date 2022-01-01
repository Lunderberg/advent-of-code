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

#[derive(Debug)]
enum Constraint {
    IsZero(Expression),
    RestrictedRange {
        var: Variable,
        min: Expression,
        max: Expression,
    },
    BooleanVar(Variable),
}

#[derive(Debug)]
struct RuntimeExprs {
    inputs: VecDeque<Expression>,
    constraints: Vec<Constraint>,
    vals: [Expression; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Variable {
    // A value in memory after the program completes, whose value is
    // unchecked.  (e.g. Memory locations W/X/Y, which are
    // unconstrained in part 1.)
    FinalState {
        loc: MemoryLocation,
    },

    // A value that is overwritten with an output value as part of the
    // instruction on line_num.  I think these should only occur as
    // part of Input commands, since all other instructions are of the
    // form `a = f(a,b)` and introduce a constraint on the prior value
    // of `a`.
    OverwrittenValue {
        loc: MemoryLocation,
        line_num: usize,
    },

    // A value input into the program.
    InputValue {
        input_num: usize,
    },

    // A parameter introduced while unpacking an injective operator.
    // Additional constraints may be expressed in
    // `RuntimeConstraints.parameter_constraints`.
    Parameter {
        line_num: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Expression {
    // No expression exists that satisfies the constraints.  For
    // example, backpropagating [0,*,*,*] through the instruction `eql
    // x x` would result in [Impossible, *,*,*], because no value of x
    // can produce 0.
    Impossible,

    // An unconstrained variable, introduced in back propagation.
    Variable(Variable),

    // An integer literal
    Int(i64),

    // Unary NOT.  Not(0) = 1.  Not(x) = 0 for all other x.
    Not(Box<Expression>),

    // Binary operators
    Add(Box<(Expression, Expression)>),
    Sub(Box<(Expression, Expression)>),
    Mul(Box<(Expression, Expression)>),
    Div(Box<(Expression, Expression)>),
    Mod(Box<(Expression, Expression)>),
    Equal(Box<(Expression, Expression)>),

    // Ternary operator.  Any non-zero expression is treated as true.
    IfThenElse(Box<(Expression, Expression, Expression)>),
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
    fn locations() -> [MemoryLocation; 4] {
        use MemoryLocation::*;
        [W, X, Y, Z]
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

impl RuntimeExprs {
    fn new_final_state() -> Self {
        let vals = MemoryLocation::locations()
            .map(|loc| Variable::FinalState { loc }.into());
        Self {
            vals,
            inputs: VecDeque::new(),
            constraints: Vec::new(),
        }
    }

    fn new_initial_state() -> Self {
        let vals = MemoryLocation::locations().map(|_loc| 0i64.into());
        Self {
            vals,
            inputs: VecDeque::new(),
            constraints: Vec::new(),
        }
    }

    fn get_arg(&self, arg: &Argument) -> Expression {
        match arg {
            Argument::MemLoc(loc) => self[*loc].clone(),
            Argument::Int(num) => num.into(),
        }
    }

    fn backprop_instruction(
        mut self,
        inst: &Instruction,
        line_num: usize,
    ) -> Self {
        println!("-------- Backtracking --------");
        println!("After {}", inst);
        self.vals
            .iter()
            .enumerate()
            .map(|(i, expr)| -> (MemoryLocation, _) {
                (i.try_into().unwrap(), expr)
            })
            .for_each(|(loc, expr)| println!("\t{} = {}", loc, expr));

        let loc = inst.loc();
        let arg = inst.arg().map(|arg| self.get_arg(&arg));
        let input_var = Variable::OverwrittenValue { loc, line_num };

        // Extract the expression that must be returned by this instruction.
        let output_expr = std::mem::replace(&mut self[loc], input_var.into());

        match inst {
            Instruction::Input(..) => {
                println!("Output expr = {}", output_expr);
                self.inputs.push_front(output_expr);
            }
            Instruction::Equal(..) => {
                use Expression::*;

                if let Variable(var) = output_expr {
                    println!("New constraint: {} is bool", var);
                    self.constraints.push(Constraint::BooleanVar(var));
                }

                self[loc] = match (arg.unwrap(), output_expr) {
                    (arg, Int(0)) => !arg,
                    (arg, Int(1)) => arg,
                    (_arg, Int(_)) => Expression::Impossible,
                    (Int(0), output_expr) => !output_expr,
                    (Int(1), output_expr) => output_expr,
                    (arg, output_expr) => IfThenElse(Box::new((
                        output_expr,
                        arg,
                        input_var.into(),
                    ))),
                };
            }
            Instruction::Add(..) => {
                self[loc] = output_expr - arg.unwrap();
            }
            Instruction::Mul(_loc, Argument::Int(0)) => {
                // Leave self[loc] as an undetermined value.
                println!("New constraint: {} = 0", output_expr);
                self.constraints.push(Constraint::IsZero(output_expr));
            }
            Instruction::Mul(..) => {
                self[loc] = output_expr / arg.unwrap();
            }
            Instruction::Mod(..) => {
                let n: Expression = Variable::Parameter { line_num }.into();
                self[loc] = n * arg.unwrap() + output_expr;
            }
            Instruction::Div(_loc, Argument::Int(1)) => {
                self[loc] = output_expr;
            }
            Instruction::Div(..) => {
                let arg = arg.unwrap();
                let n = Variable::Parameter { line_num }.into();

                let constraint = Constraint::RestrictedRange {
                    var: n,
                    min: 0i64.into(),
                    max: arg.clone(),
                };
                println!("New constraint: {}", constraint);
                self.constraints.push(constraint);
                self[loc] = output_expr * arg + n.into();
            }
        }
        println!("Before {}", inst);
        self.vals
            .iter()
            .enumerate()
            .map(|(i, expr)| -> (MemoryLocation, _) {
                (i.try_into().unwrap(), expr)
            })
            .for_each(|(loc, expr)| println!("\t{} = {}", loc, expr));
        self
    }

    fn apply_instruction(
        mut self,
        inst: &Instruction,
        input_num: usize,
    ) -> Self {
        println!("-------- Forward Prop --------");
        println!("Before {}", inst);
        self.vals
            .iter()
            .enumerate()
            .map(|(i, expr)| -> (MemoryLocation, _) {
                (i.try_into().unwrap(), expr)
            })
            .for_each(|(loc, expr)| println!("\t{} = {}", loc, expr));

        let loc = inst.loc();
        let a = std::mem::replace(&mut self[loc], 0i64.into());
        let b = inst
            .arg()
            .map(|arg| self.get_arg(&arg))
            .unwrap_or(0i64.into());

        self[loc] = match inst {
            Instruction::Input(..) => Variable::InputValue { input_num }.into(),
            Instruction::Equal(..) => Expression::Equal(Box::new((a, b))),
            Instruction::Add(..) => Expression::Add(Box::new((a, b))),
            Instruction::Mul(..) => Expression::Mul(Box::new((a, b))),
            Instruction::Mod(..) => Expression::Mod(Box::new((a, b))),
            Instruction::Div(..) => Expression::Div(Box::new((a, b))),
        }
        .simplify();

        println!("After {}", inst);
        self.vals
            .iter()
            .enumerate()
            .map(|(i, expr)| -> (MemoryLocation, _) {
                (i.try_into().unwrap(), expr)
            })
            .for_each(|(loc, expr)| println!("\t{} = {}", loc, expr));

        self
    }
}

impl Display for Constraint {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Constraint::*;
        match self {
            BooleanVar(var) => write!(f, "{} in [0,1]", var),
            IsZero(expr) => write!(f, "{} = 0", expr),
            RestrictedRange { var, min, max } => {
                write!(f, "{} on range [{}, {})", var, min, max)
            }
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Variable::*;
        match self {
            InputValue { input_num } => write!(f, "i{}", input_num),
            FinalState { loc } => write!(f, "{}f", loc),
            OverwrittenValue { loc, line_num } => {
                write!(f, "{}{}", loc, line_num)
            }
            Parameter { line_num } => {
                write!(f, "n{}", line_num)
            }
        }
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        use Expression::*;

        let with_paren = |node: &Expression| {
            if node.priority() > self.priority() {
                format!("({})", node)
            } else {
                format!("{}", node)
            }
        };

        match self {
            Impossible => write!(f, "!!!"),
            Int(num) => write!(f, "{}", num),
            Not(expr) => match &**expr {
                Variable { .. } => write!(f, "!{}", expr),
                Equal(boxed) => write!(f, "{} != {}", boxed.0, boxed.1),
                _ => write!(f, "!({})", expr),
            },
            Variable(var) => write!(f, "{}", var),
            Equal(boxed) => write!(f, "{} == {}", boxed.0, boxed.1),
            Add(boxed) => write!(f, "{} + {}", boxed.0, boxed.1),
            Sub(boxed) => write!(f, "{} - {}", boxed.0, boxed.1),
            Mod(boxed) => {
                let (a, b) = &**boxed;
                write!(f, "{} % {}", with_paren(a), with_paren(b))
            }
            Mul(boxed) => {
                let (a, b) = &**boxed;
                write!(f, "{} * {}", with_paren(a), with_paren(b))
            }
            Div(boxed) => {
                let (a, b) = &**boxed;
                write!(f, "{} / {}", with_paren(a), with_paren(b))
            }
            IfThenElse(boxed) => {
                let (cond, if_expr, then_expr) = &**boxed;
                write!(
                    f,
                    "{} ? {} : {}",
                    with_paren(cond),
                    with_paren(if_expr),
                    with_paren(then_expr),
                )
            }
        }
    }
}

impl ops::Not for Expression {
    type Output = Expression;
    fn not(self) -> Self {
        use Expression::*;
        match self {
            Impossible => Impossible,
            Int(a) => match a {
                0 => 1i64.into(),
                1 => 0i64.into(),
                _ => Impossible,
            },
            Not(a) => *a,
            _ => Not(Box::new(self)),
        }
    }
}

impl ops::Add for Expression {
    type Output = Expression;
    fn add(self, rhs: Self) -> Self {
        use Expression::*;
        match (self, rhs) {
            (Impossible, _) | (_, Impossible) => Impossible,
            (Int(a), Int(b)) => Int(a + b),
            (a, b) => Add(Box::new((a, b))),
        }
    }
}

impl ops::Sub for Expression {
    type Output = Expression;
    fn sub(self, rhs: Self) -> Self {
        use Expression::*;
        match (self, rhs) {
            (Impossible, _) | (_, Impossible) => Impossible,
            (Int(a), Int(b)) => Int(a - b),
            (a, b) => Sub(Box::new((a, b))),
        }
    }
}

impl ops::Mul for Expression {
    type Output = Expression;
    fn mul(self, rhs: Self) -> Self {
        use Expression::*;
        match (self, rhs) {
            (Impossible, _) | (_, Impossible) => Impossible,
            (Int(a), Int(b)) => Int(a * b),
            (a, b) => Mul(Box::new((a, b))),
        }
    }
}

impl ops::Div for Expression {
    type Output = Expression;
    fn div(self, rhs: Self) -> Self {
        use Expression::*;
        match (self, rhs) {
            (Impossible, _) | (_, Impossible) => Impossible,
            (Int(a), Int(b)) => Int(a / b),
            (a, b) => Div(Box::new((a, b))),
        }
    }
}

impl From<Variable> for Expression {
    fn from(var: Variable) -> Self {
        Self::Variable(var)
    }
}

impl From<i64> for Expression {
    fn from(num: i64) -> Self {
        Self::Int(num)
    }
}

impl From<&i64> for Expression {
    fn from(num: &i64) -> Self {
        Self::Int(*num)
    }
}

impl Expression {
    fn priority(&self) -> usize {
        use Expression::*;
        match self {
            //
            Impossible => 0,
            Variable(_) => 0,
            Int(_) => 0,
            //
            Add(_) => 10,
            Sub(_) => 10,
            //
            Mul(_) => 20,
            Mod(_) => 20,
            Not(_) => 20,
            //
            Div(_) => 30,
            //
            Equal(_) => 40,
            //
            IfThenElse(_) => 50,
        }
    }

    fn simplify(self) -> Self {
        use Expression::*;

        match self {
            Add(boxed) => match *boxed {
                (Int(a), Int(b)) => Int(a + b),
                (Int(0), b) => b,
                (a, Int(0)) => a,
                _ => Add(boxed),
            },
            Mul(boxed) => match *boxed {
                (Int(a), Int(b)) => Int(a * b),
                (Int(1), b) => b,
                (a, Int(1)) => a,
                (Int(0), _) => Int(0),
                (_, Int(0)) => Int(0),
                _ => Mul(boxed),
            },
            Div(boxed) => match *boxed {
                (Int(a), Int(b)) => Int(a / b),
                (a, Int(1)) => a,
                (Int(0), _) => Int(0),
                (_, Int(0)) => Impossible,
                _ => Div(boxed),
            },
            Mod(boxed) => match *boxed {
                (Int(a), Int(b)) => Int(a % b),
                (_, Int(0)) => Impossible,
                (_a, Int(1)) => Int(0),
                (Int(0), _) => Int(0),
                //(Int(1), _) => Int(1),
                _ => Mod(boxed),
            },
            Equal(boxed) => match *boxed {
                (Int(a), Int(b)) => Int((a == b) as i64),
                (a, Int(0)) => Not(Box::new(a)),
                (Int(0), b) => Not(Box::new(b)),
                _ => Equal(boxed),
            },
            Not(boxed) => match *boxed {
                Int(a) => Int((a == 0) as i64),
                _ => Not(boxed),
            },
            _ => self,
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

impl ops::Index<MemoryLocation> for RuntimeExprs {
    type Output = Expression;
    fn index(&self, var: MemoryLocation) -> &Expression {
        &self.vals[var.index()]
    }
}

impl ops::IndexMut<MemoryLocation> for RuntimeExprs {
    fn index_mut(&mut self, var: MemoryLocation) -> &mut Expression {
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

    fn backprop_instructions(&self, constraints: RuntimeExprs) -> RuntimeExprs {
        self.instructions.iter().enumerate().rev().fold(
            constraints,
            |state, (line_num, inst)| {
                state.backprop_instruction(inst, line_num)
            },
        )
    }

    fn prop_inputs(&self) -> RuntimeExprs {
        self.instructions
            .iter()
            .scan(0, |num_inputs, inst| {
                if let Instruction::Input(_) = inst {
                    *num_inputs += 1;
                }
                Some((*num_inputs - 1, inst))
            })
            .fold(
                RuntimeExprs::new_initial_state(),
                |state, (input_num, inst)| {
                    state.apply_instruction(inst, input_num)
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

        //println!("Program:\n{}", program);

        {
            let serial: i64 = 13579246899999;
            let num_digits = (serial as f64).log10().ceil() as u32;
            let digits =
                (0..num_digits).rev().map(|i| (serial / 10_i64.pow(i)) % 10);

            let _result = program.execute(digits);
        }

        // {
        //     let mut final_constraints = RuntimeExprs::new();
        //     final_constraints[MemoryLocation::Z] = 0i64.into();

        //     let initial_constraints =
        //         program.backprop_instructions(final_constraints);
        //     initial_constraints
        //         .constraints
        //         .iter()
        //         .for_each(|constraint| println!("{}", constraint));
        //     initial_constraints
        //         .inputs
        //         .iter()
        //         .enumerate()
        //         .for_each(|(i, expr)| println!("Input #{} is {}", i, expr));
        // }

        {
            let state = program.prop_inputs();
            println!("Final state: {:?}", state);
        }

        let result = ();
        Ok(Box::new(result))
    }
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error> {
        let result = ();
        Ok(Box::new(result))
    }
}
