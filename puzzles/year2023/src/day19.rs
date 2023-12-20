use std::{collections::HashMap, fmt::Display, ops::Range, str::FromStr};

use aoc_utils::prelude::*;

#[derive(Debug)]
enum Goto {
    Accepted,
    Rejected,
    Jump(String),
}

#[derive(Debug, Clone, Copy)]
enum Parameter {
    X,
    M,
    A,
    S,
}

#[derive(Debug)]
pub struct MachinePart([u32; 4]);

#[derive(Debug, Clone)]
pub struct MachinePartSet([Range<u32>; 4]);

#[derive(Debug)]
enum Operator {
    GreaterThan,
    LessThan,
}

#[derive(Debug)]
struct Boolean {
    param: Parameter,
    op: Operator,
    threshold: u32,
}

#[derive(Debug)]
struct Condition {
    cond: Option<Boolean>,
    target: Goto,
}

#[derive(Debug)]
struct Workflow {
    conditions: Vec<Condition>,
}

#[derive(Debug)]
pub struct System {
    start: Goto,
    workflows: HashMap<String, Workflow>,
}

impl FromStr for Workflow {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let conditions = s
            .split(',')
            .map(|cond| cond.parse())
            .collect::<Result<_, _>>()?;
        Ok(Workflow { conditions })
    }
}
impl FromStr for Condition {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.rsplit(':');

        let target = parts
            .next()
            .ok_or_else(|| Error::InvalidString(s.to_string()))?
            .parse()?;
        let cond = parts.next().map(|cond_str| cond_str.parse()).transpose()?;

        if parts.next().is_none() {
            Ok(Self { cond, target })
        } else {
            Err(Error::InvalidString(s.to_string()))
        }
    }
}
impl FromStr for Goto {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "A" => Goto::Accepted,
            "R" => Goto::Rejected,
            _ => Goto::Jump(s.to_string()),
        })
    }
}
impl FromStr for Boolean {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let param = s[..1].parse()?;
        let op = s[1..2].parse()?;
        let threshold = s[2..].parse()?;
        Ok(Boolean {
            param,
            op,
            threshold,
        })
    }
}
impl FromStr for Parameter {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x" => Ok(Parameter::X),
            "m" => Ok(Parameter::M),
            "a" => Ok(Parameter::A),
            "s" => Ok(Parameter::S),
            _ => Err(Error::InvalidString(s.to_string())),
        }
    }
}
impl FromStr for Operator {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "<" => Ok(Operator::LessThan),
            ">" => Ok(Operator::GreaterThan),
            _ => Err(Error::InvalidString(s.to_string())),
        }
    }
}
impl FromStr for MachinePart {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let last_index = s.len() - 1;
        if &s[..1] == "{" && &s[last_index..] == "}" {
            s[1..last_index]
                .split(',')
                .map(|setter| -> Result<(Parameter, u32), Error> {
                    let param = setter[..1].parse()?;
                    let value = setter[2..].parse()?;
                    Ok((param, value))
                })
                .try_fold(MachinePart([0u32; 4]), |mut state, res| {
                    let (param, value) = res?;
                    state[param] = value;
                    Ok(state)
                })
        } else {
            Err(Error::InvalidString(s.to_string()))
        }
    }
}

impl std::ops::Index<Parameter> for MachinePart {
    type Output = u32;

    fn index(&self, param: Parameter) -> &Self::Output {
        &self.0[param.as_index()]
    }
}
impl std::ops::IndexMut<Parameter> for MachinePart {
    fn index_mut(&mut self, param: Parameter) -> &mut Self::Output {
        &mut self.0[param.as_index()]
    }
}
impl std::ops::Index<Parameter> for MachinePartSet {
    type Output = Range<u32>;

    fn index(&self, param: Parameter) -> &Self::Output {
        &self.0[param.as_index()]
    }
}
impl std::ops::IndexMut<Parameter> for MachinePartSet {
    fn index_mut(&mut self, param: Parameter) -> &mut Self::Output {
        &mut self.0[param.as_index()]
    }
}

impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Parameter::X => 'X',
            Parameter::M => 'M',
            Parameter::A => 'A',
            Parameter::S => 'S',
        };
        write!(f, "{c}")
    }
}
impl Display for MachinePartSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        Parameter::iter_all()
            .enumerate()
            .try_for_each(|(i, param)| {
                if i > 0 {
                    write!(f, ", ")?;
                }
                let Range { start, end } = &self[param];
                write!(f, "{start} <= {param} < {end}")
            })?;
        write!(f, "}}")
    }
}

impl Parameter {
    fn as_index(self) -> usize {
        match self {
            Parameter::X => 0,
            Parameter::M => 1,
            Parameter::A => 2,
            Parameter::S => 3,
        }
    }

    fn iter_all() -> impl Iterator<Item = Self> {
        [Parameter::X, Parameter::M, Parameter::A, Parameter::S].into_iter()
    }
}

impl System {
    fn sort_part<'a>(&'a self, part: &MachinePart) -> Result<&'a Goto, Error> {
        std::iter::successors(Some(Ok(&self.start)), |prev| match prev {
            Ok(Goto::Jump(target)) => {
                Some(self.workflows.get(target)?.sort_part(part))
            }
            _ => None,
        })
        .last()
        .unwrap()
    }

    fn iter_sets(
        &self,
        set: MachinePartSet,
    ) -> impl Iterator<Item = MachinePartSet> + '_ {
        self.workflows["in"].iter_sets(set, self)
    }
}
impl Workflow {
    fn sort_part<'a>(&'a self, part: &MachinePart) -> Result<&'a Goto, Error> {
        self.conditions
            .iter()
            .find_map(|condition| condition.apply(part))
            .ok_or(Error::NoneError)
    }

    fn iter_sets<'a>(
        &'a self,
        set: MachinePartSet,
        system: &'a System,
    ) -> impl Iterator<Item = MachinePartSet> + 'a {
        self.conditions
            .iter()
            .scan(set, |state, cond| {
                let (pass, fail) = cond.split(state);
                *state = fail;
                Some((pass, &cond.target))
            })
            .filter(|(subset, _)| !subset.is_empty())
            .flat_map(move |(subset, goto)| -> Box<dyn Iterator<Item=MachinePartSet> + '_>   {
                match goto {
                  Goto::Accepted => {
                    Box::new(std::iter::once(subset))
                },
                Goto::Rejected => {
                    Box::new(std::iter::empty())},
                Goto::Jump(workflow) => {
                    Box::new(system.workflows[workflow].iter_sets(subset, system))
                }
            }})
    }
}
impl Condition {
    fn apply<'a>(&'a self, part: &MachinePart) -> Option<&'a Goto> {
        self.cond
            .as_ref()
            .map_or(true, |cond| cond.eval(part))
            .then(|| &self.target)
    }

    fn split(&self, set: &MachinePartSet) -> (MachinePartSet, MachinePartSet) {
        self.cond
            .as_ref()
            .map(|c| c.split(set))
            .unwrap_or_else(|| (set.clone(), MachinePartSet::new_empty()))
    }
}
impl Boolean {
    fn eval(&self, part: &MachinePart) -> bool {
        let value = part[self.param];
        match self.op {
            Operator::GreaterThan => value > self.threshold,
            Operator::LessThan => value < self.threshold,
        }
    }

    fn split(&self, set: &MachinePartSet) -> (MachinePartSet, MachinePartSet) {
        let mut pass = set.clone();
        let mut fail = set.clone();
        match self.op {
            Operator::GreaterThan => {
                {
                    let range = &mut pass[self.param];
                    range.start = range.start.max(self.threshold + 1);
                }
                {
                    let range = &mut fail[self.param];
                    range.end = range.end.min(self.threshold + 1);
                }
            }
            Operator::LessThan => {
                {
                    let range = &mut pass[self.param];
                    range.end = range.end.min(self.threshold);
                }
                {
                    let range = &mut fail[self.param];
                    range.start = range.start.max(self.threshold);
                }
            }
        }
        (pass, fail)
    }
}
impl MachinePart {
    fn sum(&self) -> u32 {
        self.0.iter().sum()
    }
}
impl MachinePartSet {
    fn count(&self) -> u64 {
        self.0
            .iter()
            .map(|range| (range.end - range.start) as u64)
            .product()
    }

    fn is_empty(&self) -> bool {
        self.0.iter().all(|range| range.is_empty())
    }

    fn new_empty() -> Self {
        Self([(); 4].map(|_| 0..0))
    }
}

impl Default for MachinePartSet {
    fn default() -> Self {
        Self([(); 4].map(|_| 1..4001))
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = (System, Vec<MachinePart>);
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let workflows = lines
            .by_ref()
            .take_while(|line| !line.is_empty())
            .map(|line| -> Result<_, Error> {
                let (name, workflow) = line
                    .split('{')
                    .collect_tuple()
                    .ok_or(Error::WrongIteratorSize)?;
                let name = name.to_string();
                let workflow = workflow.trim_end_matches('}').parse()?;
                Ok((name, workflow))
            })
            .collect::<Result<_, _>>()?;
        let parts = lines.map(|line| line.parse()).collect::<Result<_, _>>()?;
        Ok((
            System {
                start: Goto::Jump("in".to_string()),
                workflows,
            },
            parts,
        ))
    }

    fn part_1(
        (system, parts): &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = parts
            .iter()
            .map(|part| system.sort_part(part).map(|sorted| (sorted, part)))
            .filter_ok(|(sorted, _)| matches!(sorted, Goto::Accepted))
            .map_ok(|(_, part)| part.sum())
            .sum::<Result<u32, _>>()?;
        Ok(value)
    }

    fn part_2(
        (system, _): &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = system
            .iter_sets(Default::default())
            .map(|part_set| part_set.count())
            .sum::<u64>();
        Ok(value)
    }
}
