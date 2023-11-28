#![allow(unused_imports)]
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    str::FromStr,
};

use crate::{Error, Puzzle};

use itertools::Itertools;

pub struct Circuit {
    wires: Vec<Connection>,
}

enum Arg {
    Const(u16),
    Ref(String),
}
enum Expr {
    Direct(Arg),
    And(Arg, Arg),
    Or(Arg, Arg),
    Not(Arg),
    LeftShift(Arg, Arg),
    RightShift(Arg, Arg),
}
struct Connection {
    input: Expr,
    output: String,
}

impl Arg {
    fn iter_ref(&self) -> impl Iterator<Item = &str> + '_ {
        match self {
            Arg::Const(_) => None,
            Arg::Ref(name) => Some(name.as_str()),
        }
        .into_iter()
    }

    fn eval(&self, known: &HashMap<String, u16>) -> Result<u16, Error> {
        let value = match self {
            Arg::Const(val) => val,
            Arg::Ref(name) => known
                .get(name)
                .ok_or_else(|| Error::MissingValue(name.clone()))?,
        };
        Ok(*value)
    }
}

impl Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arg::Const(val) => write!(f, "{val}"),
            Arg::Ref(name) => write!(f, "{name}"),
        }
    }
}

impl Expr {
    fn iter_arg(&self) -> impl Iterator<Item = &Arg> + '_ {
        match self {
            Expr::Direct(a) => [Some(a), None],
            Expr::And(a, b) => [Some(a), Some(b)],
            Expr::Or(a, b) => [Some(a), Some(b)],
            Expr::Not(a) => [Some(a), None],
            Expr::LeftShift(a, b) => [Some(a), Some(b)],
            Expr::RightShift(a, b) => [Some(a), Some(b)],
        }
        .into_iter()
        .flatten()
    }

    fn iter_ref(&self) -> impl Iterator<Item = &str> + '_ {
        self.iter_arg().flat_map(|arg| arg.iter_ref())
    }

    fn eval(&self, known: &HashMap<String, u16>) -> Result<u16, Error> {
        let value = match self {
            Expr::Direct(arg) => arg.eval(known)?,
            Expr::And(a, b) => a.eval(known)? & b.eval(known)?,
            Expr::Or(a, b) => a.eval(known)? | b.eval(known)?,
            Expr::Not(a) => !a.eval(known)?,
            Expr::LeftShift(a, b) => {
                a.eval(known)?.wrapping_shl(b.eval(known)?.into())
            }
            Expr::RightShift(a, b) => {
                a.eval(known)?.wrapping_shr(b.eval(known)?.into())
            }
        };
        Ok(value)
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Direct(arg) => write!(f, "{arg}"),
            Expr::And(a, b) => write!(f, "{a} AND {b}"),
            Expr::Or(a, b) => write!(f, "{a} OR {b}"),
            Expr::Not(a) => write!(f, "NOT {a}"),
            Expr::LeftShift(a, b) => write!(f, "{a} LSHIFT {b}"),
            Expr::RightShift(a, b) => write!(f, "{a} RSHIFT {b}"),
        }
    }
}

impl FromStr for Arg {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().all(|c| c.is_ascii_digit()) {
            Ok(Arg::Const(s.parse()?))
        } else if s.chars().all(|c| c.is_ascii_alphabetic()) {
            Ok(Arg::Ref(s.to_string()))
        } else {
            Err(Error::UnexpectedToken(s.to_string()))
        }
    }
}

impl FromStr for Connection {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut tokens = s.split_ascii_whitespace().peekable();

        let input = if let Some(_) = tokens.next_if(|s| s == &"NOT") {
            let arg: Arg =
                tokens.next().ok_or(Error::UnexpectedEndOfStream)?.parse()?;
            Expr::Not(arg)
        } else {
            let lhs: Arg =
                tokens.next().ok_or(Error::UnexpectedEndOfStream)?.parse()?;
            if let Some(&"->") = tokens.peek() {
                Expr::Direct(lhs)
            } else {
                let op = tokens.next().ok_or(Error::UnexpectedEndOfStream)?;
                let rhs: Arg = tokens
                    .next()
                    .ok_or(Error::UnexpectedEndOfStream)?
                    .parse()?;
                match op {
                    "AND" => Ok(Expr::And(lhs, rhs)),
                    "OR" => Ok(Expr::Or(lhs, rhs)),
                    "LSHIFT" => Ok(Expr::LeftShift(lhs, rhs)),
                    "RSHIFT" => Ok(Expr::RightShift(lhs, rhs)),
                    _ => Err(Error::UnexpectedToken(op.to_string())),
                }?
            }
        };

        let assignment = tokens.next().ok_or(Error::UnexpectedEndOfStream)?;
        match assignment {
            "->" => Ok(()),
            _ => Err(Error::UnexpectedToken(assignment.to_string())),
        }?;

        let output = tokens
            .next()
            .ok_or(Error::UnexpectedEndOfStream)?
            .to_string();
        if let Some(after_end) = tokens.next() {
            Err(Error::UnexpectedToken(after_end.to_string()))
        } else {
            Ok(Self { input, output })
        }
    }
}

impl Display for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.input, self.output)
    }
}

impl Circuit {
    fn topological_sort(self) -> Result<Self, Error> {
        let mut old_wires = self.wires;
        let mut new_wires = Vec::new();
        let mut sorted = HashSet::<String>::new();

        loop {
            let n_before = old_wires.len();
            old_wires = old_wires
                .into_iter()
                .filter_map(|wire| {
                    let all_inputs_computed =
                        wire.input.iter_ref().all(|conn| sorted.contains(conn));
                    if all_inputs_computed {
                        sorted.insert(wire.output.clone());
                        new_wires.push(wire);
                        None
                    } else {
                        Some(wire)
                    }
                })
                .collect();
            let n_after = old_wires.len();

            if n_after == 0 {
                return Ok(Circuit { wires: new_wires });
            } else if n_after == n_before {
                return Err(Error::CycleDetected);
            }
        }
    }

    /// Evaluate the circuit, potentially with overrides
    fn eval_into(
        &self,
        mut values: HashMap<String, u16>,
    ) -> Result<HashMap<String, u16>, Error> {
        for conn in &self.wires {
            if !values.contains_key(&conn.output) {
                let value = conn.input.eval(&values)?;
                values.insert(conn.output.clone(), value);
            }
        }
        Ok(values)
    }

    /// Evaluate the circuit as specified
    fn eval(&self) -> Result<HashMap<String, u16>, Error> {
        self.eval_into(HashMap::new())
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Circuit;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let wires: Result<_, _> = lines.map(|line| line.parse()).collect();
        let circuit = Circuit { wires: wires? };
        let circuit = circuit.topological_sort()?;
        Ok(circuit)
    }

    fn part_1(
        circuit: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let values = circuit.eval()?;
        let value = values["a"];
        Ok(value)
    }

    fn part_2(
        circuit: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let orig = circuit.eval()?;
        let override_values = [("b".to_string(), orig["a"])].into();
        let values = circuit.eval_into(override_values)?;
        let value = values["a"];
        Ok(value)
    }
}
