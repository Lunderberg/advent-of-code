use aoc_utils::prelude::*;

use std::cmp::Reverse;

#[derive(Debug, Clone)]
pub struct CraneYard {
    stacks: Vec<Stack>,
}

#[derive(Debug, Clone)]
pub struct Stack {
    contents: Vec<char>,
}

#[derive(Debug)]
pub struct Instruction {
    move_from: usize,
    move_to: usize,
    num_to_move: usize,
}

impl CraneYard {
    fn stacks_affected(
        &mut self,
        inst: &Instruction,
    ) -> Result<(&mut Vec<char>, &mut Vec<char>), Error> {
        // let stack_from = &mut self.stacks[inst.move_from].contents;
        // let stack_to = &mut self.stacks[inst.move_to].contents;

        // Has to be a better way to mutably borrow multiple elements.
        Ok(self
            .stacks
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| *i == inst.move_from || *i == inst.move_to)
            .sorted_by_key(|(i, _)| *i == inst.move_from)
            .map(|(_i, stack)| &mut stack.contents)
            .tuples()
            .exactly_one_or_err()?)
    }

    fn apply_part1(&mut self, inst: &Instruction) -> Result<(), Error> {
        let (stack_to, stack_from) = self.stacks_affected(inst)?;

        (0..inst.num_to_move).try_for_each(|_| -> Result<(), Error> {
            let value = stack_from.pop().ok_or(Error::NoneError)?;
            stack_to.push(value);
            Ok(())
        })
    }

    fn apply_part2(&mut self, inst: &Instruction) -> Result<(), Error> {
        let (stack_to, stack_from) = self.stacks_affected(inst)?;

        let moved_crates = (0..inst.num_to_move)
            .map(|_| stack_from.pop().ok_or(Error::NoneError))
            .collect::<Result<Vec<_>, _>>()?;

        moved_crates
            .into_iter()
            .rev()
            .for_each(|value| stack_to.push(value));

        Ok(())
    }

    fn top(&self) -> Result<String, Error> {
        self.stacks
            .iter()
            .map(|stack| stack.contents.last().ok_or(Error::NoneError))
            .collect()
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = (CraneYard, Vec<Instruction>);
    fn parse_input<'a>(
        mut lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let stacks = lines
            .by_ref()
            .take_while(|line| !line.is_empty())
            .enumerate()
            .flat_map(|(i, line)| {
                line.chars().skip(1).step_by(4).enumerate().map(
                    move |(col, c)| -> (usize, usize, char) { (i, col, c) },
                )
            })
            .filter(|(_i, _col, c)| c.is_ascii_uppercase())
            .sorted_by_key(|(i, _col, _c)| Reverse(*i))
            .map(|(_i, col, c)| (col, c))
            .into_group_map()
            .into_iter()
            .map(|(col, contents)| (col, Stack { contents }))
            .sorted_by_key(|(col, _stack)| *col)
            .map(|(_col, stack)| stack)
            .collect();

        let craneyard = CraneYard { stacks };

        let instructions = lines
            .map(|line| -> Result<Instruction, Error> {
                let (num_to_move, move_from, move_to) = line
                    .split_ascii_whitespace()
                    .skip(1)
                    .step_by(2)
                    .map(|s| s.parse::<usize>())
                    .tuples()
                    .exactly_one_or_err()?;
                Ok(Instruction {
                    move_from: move_from? - 1,
                    move_to: move_to? - 1,
                    num_to_move: num_to_move?,
                })
            })
            .collect::<Result<_, _>>()?;

        Ok((craneyard, instructions))
    }

    fn part_1(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let (craneyard, instructions) = values;
        let mut craneyard = craneyard.clone();

        instructions
            .iter()
            .try_for_each(|inst| craneyard.apply_part1(inst))?;

        craneyard.top()
    }

    fn part_2(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let (craneyard, instructions) = values;
        let mut craneyard = craneyard.clone();

        instructions
            .iter()
            .try_for_each(|inst| craneyard.apply_part2(inst))?;

        craneyard.top()
    }
}
