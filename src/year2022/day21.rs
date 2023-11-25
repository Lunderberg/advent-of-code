#![allow(unused_imports)]
use crate::{Error, Puzzle};

use itertools::Itertools;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct MonkeySystem {
    monkeys: Vec<Monkey>,
}

#[derive(Debug, Clone)]
struct Monkey {
    name: String,
    op: Operation,
}

#[derive(Debug, Clone)]
struct MonkeyNode<'a> {
    system: &'a MonkeySystem,
    index: usize,
}

#[derive(Debug, Clone, Copy)]
enum Operation {
    Num(i64),
    Human,
    Add(usize, usize),
    Sub(usize, usize),
    Mul(usize, usize),
    Div(usize, usize),
    Equal(usize, usize),
}

struct MonkeySpec {
    name: String,
    op: OperationSpec,
}

enum OperationSpec {
    Num(i64),
    Add(String, String),
    Sub(String, String),
    Mul(String, String),
    Div(String, String),
}

impl std::str::FromStr for MonkeySpec {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let words: Vec<_> = s.split_ascii_whitespace().collect();
        let name = words
            .first()
            .and_then(|word| word.strip_suffix(':'))
            .ok_or_else(|| Error::InvalidString(s.to_string()))?
            .to_string();
        let op = if words.len() == 4 {
            let a = words[1].to_string();
            let b = words[3].to_string();
            let c = words[2].chars().exactly_one()?;
            match c {
                '+' => Ok(OperationSpec::Add(a, b)),
                '-' => Ok(OperationSpec::Sub(a, b)),
                '*' => Ok(OperationSpec::Mul(a, b)),
                '/' => Ok(OperationSpec::Div(a, b)),
                _ => Err(Error::UnknownChar(c)),
            }
        } else if words.len() == 2 {
            let val: i64 = words[1].parse()?;
            Ok(OperationSpec::Num(val))
        } else {
            Err(Error::InvalidString(s.to_string()))
        }?;
        Ok(Self { name, op })
    }
}

impl Operation {
    fn precedence(&self) -> usize {
        match self {
            Operation::Num(_) => 5,
            Operation::Human => 5,
            Operation::Equal(_, _) => 1,
            Operation::Add(_, _) => 2,
            Operation::Sub(_, _) => 2,
            Operation::Mul(_, _) => 3,
            Operation::Div(_, _) => 3,
        }
    }
}

impl MonkeySystem {
    fn root(&self) -> Option<usize> {
        self.find_name("root")
    }

    fn find_name(&self, name: &str) -> Option<usize> {
        self.monkeys
            .iter()
            .enumerate()
            .find_map(|(i, monkey)| (monkey.name == name).then_some(i))
    }

    fn eval(&self, monkey: usize) -> i64 {
        match self.monkeys[monkey].op {
            Operation::Num(val) => val,
            Operation::Add(a, b) => self.eval(a) + self.eval(b),
            Operation::Sub(a, b) => self.eval(a) - self.eval(b),
            Operation::Mul(a, b) => self.eval(a) * self.eval(b),
            Operation::Div(a, b) => self.eval(a) / self.eval(b),
            Operation::Human => panic!("Human not allowed here"),
            Operation::Equal(_, _) => panic!("Equal not allowed here"),
        }
    }

    fn update_for_part_2(mut self) -> Result<Self, Error> {
        let root = self.find_name("root").unwrap();
        self.monkeys[root].op = match self.monkeys[root].op {
            Operation::Add(a, b) => Operation::Equal(a, b),
            Operation::Sub(a, b) => Operation::Equal(a, b),
            Operation::Mul(a, b) => Operation::Equal(a, b),
            Operation::Div(a, b) => Operation::Equal(a, b),
            Operation::Equal(a, b) => Operation::Equal(a, b),
            Operation::Num(_) => panic!("root node shouldn't be num"),
            Operation::Human => panic!("root node shouldn't be humn"),
        };

        let human = self.find_name("humn").unwrap();
        self.monkeys[human].op = Operation::Human;

        Ok(self)
    }

    fn topological_sort(&self) -> Self {
        fn visit(
            old_index: usize,
            remap: &mut HashMap<usize, usize>,
            input: &Vec<Monkey>,
            output: &mut Vec<Option<Monkey>>,
        ) -> usize {
            if let Some(&new_index) = remap.get(&old_index) {
                new_index
            } else {
                let new_index = output.len();
                output.push(None);
                remap.insert(old_index, new_index);

                let visiting: &Monkey = &input[old_index];
                let op: Operation = match visiting.op {
                    Operation::Num(val) => Operation::Num(val),
                    Operation::Add(a, b) => Operation::Add(
                        visit(a, remap, input, output),
                        visit(b, remap, input, output),
                    ),
                    Operation::Sub(a, b) => Operation::Sub(
                        visit(a, remap, input, output),
                        visit(b, remap, input, output),
                    ),
                    Operation::Mul(a, b) => Operation::Mul(
                        visit(a, remap, input, output),
                        visit(b, remap, input, output),
                    ),
                    Operation::Div(a, b) => Operation::Div(
                        visit(a, remap, input, output),
                        visit(b, remap, input, output),
                    ),
                    Operation::Equal(a, b) => Operation::Equal(
                        visit(a, remap, input, output),
                        visit(b, remap, input, output),
                    ),
                    Operation::Human => Operation::Human,
                };
                output[new_index] = Some(Monkey {
                    name: visiting.name.clone(),
                    op,
                });

                new_index
            }
        }

        let mut remap: HashMap<usize, usize> = HashMap::new();
        let mut monkeys: Vec<Option<Monkey>> = Vec::new();
        let root = self.root().unwrap();

        visit(root, &mut remap, &self.monkeys, &mut monkeys);

        let monkeys = monkeys.into_iter().map(|opt| opt.unwrap()).collect();

        Self { monkeys }
    }

    fn simplify(mut self) -> Self {
        #[derive(Debug)]
        enum VisitResult {
            FullySimplified,
            TryAgainLater,
            SimplifiedVal(i64),
            SimplifiedEqual(usize, i64),
        }

        let mut to_visit: VecDeque<_> = (0..self.monkeys.len()).collect();
        let mut iter_since_update: usize = 0;

        while let Some(visiting_i) = to_visit.pop_back() {
            use Operation::*;
            let equivalent: VisitResult = match self.monkeys[visiting_i].op {
                Num(_) | Human => VisitResult::FullySimplified,

                Add(a, b) => self
                    .get_known_value(a)
                    .zip(self.get_known_value(b))
                    .map(|(a, b)| VisitResult::SimplifiedVal(a + b))
                    .unwrap_or(VisitResult::TryAgainLater),

                Sub(a, b) => self
                    .get_known_value(a)
                    .zip(self.get_known_value(b))
                    .map(|(a, b)| VisitResult::SimplifiedVal(a - b))
                    .unwrap_or(VisitResult::TryAgainLater),

                Mul(a, b) => self
                    .get_known_value(a)
                    .zip(self.get_known_value(b))
                    .map(|(a, b)| VisitResult::SimplifiedVal(a * b))
                    .unwrap_or(VisitResult::TryAgainLater),

                Div(a, b) => self
                    .get_known_value(a)
                    .zip(self.get_known_value(b))
                    .map(|(a, b)| VisitResult::SimplifiedVal(a / b))
                    .unwrap_or(VisitResult::TryAgainLater),

                Equal(lhs, rhs) => {
                    match (self.monkeys[lhs].op, self.monkeys[rhs].op) {
                        (Div(a, b), Num(eq)) => self
                            .get_known_value(b)
                            .map(|b| VisitResult::SimplifiedEqual(a, eq * b))
                            .unwrap_or(VisitResult::TryAgainLater),

                        (Add(a, b), Num(eq)) => self
                            .get_known_value(b)
                            .map(|b| VisitResult::SimplifiedEqual(a, eq - b))
                            .or_else(|| {
                                self.get_known_value(a).map(|a| {
                                    VisitResult::SimplifiedEqual(b, eq - a)
                                })
                            })
                            .unwrap_or(VisitResult::TryAgainLater),

                        (Mul(a, b), Num(eq)) => self
                            .get_known_value(b)
                            .map(|b| VisitResult::SimplifiedEqual(a, eq / b))
                            .or_else(|| {
                                self.get_known_value(a).map(|a| {
                                    VisitResult::SimplifiedEqual(b, eq / a)
                                })
                            })
                            .unwrap_or(VisitResult::TryAgainLater),

                        (Sub(a, b), Num(eq)) => self
                            .get_known_value(b)
                            .map(|b| VisitResult::SimplifiedEqual(a, eq + b))
                            .or_else(|| {
                                self.get_known_value(a).map(|a| {
                                    VisitResult::SimplifiedEqual(b, a - eq)
                                })
                            })
                            .unwrap_or(VisitResult::TryAgainLater),

                        _ => VisitResult::TryAgainLater,
                    }
                }
            };

            match equivalent {
                VisitResult::FullySimplified => {
                    // println!(
                    //     "{} is fully simplified, no longer visiting",
                    //     MonkeyNode {
                    //         system: &self,
                    //         index: visiting_i
                    //     }
                    // );
                    iter_since_update = 0;
                }
                VisitResult::TryAgainLater => {
                    // println!(
                    //     "Nothing found for {}, continuing",
                    //     MonkeyNode {
                    //         system: &self,
                    //         index: visiting_i
                    //     }
                    // );
                    to_visit.push_front(visiting_i);
                    iter_since_update += 1;
                    if iter_since_update > to_visit.len() {
                        break;
                    }
                }
                VisitResult::SimplifiedVal(val) => {
                    // println!(
                    //     "{} simplifies into {val}, replacing",
                    //     MonkeyNode {
                    //         system: &self,
                    //         index: visiting_i
                    //     }
                    // );
                    self.monkeys[visiting_i].op = Operation::Num(val);
                    to_visit.push_front(visiting_i);
                    iter_since_update = 0;
                }
                VisitResult::SimplifiedEqual(index, val) => {
                    let new_index = self.monkeys.len();
                    self.monkeys.push(Monkey {
                        name: "simplified".to_string(),
                        op: Operation::Num(val),
                    });
                    self.monkeys[visiting_i].op =
                        Operation::Equal(index, new_index);
                    to_visit.push_back(visiting_i);
                    iter_since_update = 0;
                }
            }
        }

        self
    }

    fn get_known_value(&self, i: usize) -> Option<i64> {
        match self.monkeys[i].op {
            Operation::Num(val) => Some(val),
            _ => None,
        }
    }
}

impl<'a> MonkeyNode<'a> {
    fn precedence(&self) -> usize {
        self.system.monkeys[self.index].op.precedence()
    }
}

impl Display for MonkeySystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            MonkeyNode {
                system: self,
                index: self.root().unwrap()
            }
        )
    }
}

impl<'a> Display for MonkeyNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let visiting: &Monkey = &self.system.monkeys[self.index];
        let (lhs, rhs, op) = match visiting.op {
            Operation::Add(a, b) => (a, b, " + "),
            Operation::Sub(a, b) => (a, b, " - "),
            Operation::Mul(a, b) => (a, b, "*"),
            Operation::Div(a, b) => (a, b, "/"),
            Operation::Equal(a, b) => (a, b, " == "),
            Operation::Num(val) => {
                return write!(f, "{val}");
            }
            Operation::Human => {
                return write!(f, "Human");
            }
        };

        let lhs = MonkeyNode {
            system: self.system,
            index: lhs,
        };
        if lhs.precedence() <= self.precedence() {
            write!(f, "({lhs})")?;
        } else {
            write!(f, "{lhs}")?;
        }

        write!(f, "{op}")?;

        let rhs = MonkeyNode {
            system: self.system,
            index: rhs,
        };
        if rhs.precedence() < self.precedence() {
            write!(f, "({rhs})")
        } else {
            write!(f, "{rhs}")
        }
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = MonkeySystem;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let specs: Vec<MonkeySpec> = lines
            .map(|line| line.parse())
            .collect::<Result<Vec<_>, _>>()?;

        let lookup: HashMap<String, usize> = specs
            .iter()
            .map(|spec| spec.name.clone())
            .enumerate()
            .map(|(i, name)| (name, i))
            .collect();
        let monkeys = specs
            .into_iter()
            .map(|spec| {
                let op = match spec.op {
                    OperationSpec::Num(val) => Operation::Num(val),
                    OperationSpec::Add(a, b) => Operation::Add(
                        lookup.get(&a).copied().unwrap(),
                        lookup.get(&b).copied().unwrap(),
                    ),
                    OperationSpec::Sub(a, b) => Operation::Sub(
                        lookup.get(&a).copied().unwrap(),
                        lookup.get(&b).copied().unwrap(),
                    ),
                    OperationSpec::Mul(a, b) => Operation::Mul(
                        lookup.get(&a).copied().unwrap(),
                        lookup.get(&b).copied().unwrap(),
                    ),
                    OperationSpec::Div(a, b) => Operation::Div(
                        lookup.get(&a).copied().unwrap(),
                        lookup.get(&b).copied().unwrap(),
                    ),
                };
                Monkey {
                    name: spec.name,
                    op,
                }
            })
            .collect();

        Ok(MonkeySystem { monkeys })
    }

    type Part1Result = i64;
    fn part_1(system: &Self::ParsedInput) -> Result<Self::Part1Result, Error> {
        let system = system.topological_sort();
        let root = system.root().unwrap();
        let value = system.eval(root);
        Ok(value)
    }

    type Part2Result = i64;
    fn part_2(system: &Self::ParsedInput) -> Result<Self::Part2Result, Error> {
        let system = system.clone().update_for_part_2()?;
        let system = system.topological_sort().simplify();
        let root: &Monkey = &system.monkeys[system.root().unwrap()];

        if let Operation::Equal(lhs, rhs) = root.op {
            Some((lhs, rhs))
        } else {
            None
        }
        .and_then(|(lhs, rhs)| {
            matches!(system.monkeys[lhs].op, Operation::Human).then(|| rhs)
        })
        .and_then(|rhs| {
            if let Operation::Num(val) = system.monkeys[rhs].op {
                Some(val)
            } else {
                None
            }
        })
        .ok_or_else(|| Error::NotFullySimplified(format!("{system}")))
    }
}
