use aoc_utils::prelude::*;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum OpCode {
    NoOp,
    AddX(i32),
}

impl std::str::FromStr for OpCode {
    type Err = Error;
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let (name, arg) = line
            .split_whitespace()
            .map(Some)
            .chain(std::iter::repeat(None))
            .take(2)
            .tuples()
            .exactly_one_or_err()?;

        let name = name.unwrap();
        match name {
            "noop" => Ok(OpCode::NoOp),
            "addx" => {
                Ok(OpCode::AddX(arg.ok_or(Error::NotEnoughValues)?.parse()?))
            }
            _ => Err(Error::InvalidString(name.to_string())),
        }
    }
}

fn register_x_during_cycle(
    instructions: impl Iterator<Item = OpCode>,
) -> impl Iterator<Item = i32> {
    std::iter::once(1).chain(
        instructions
            .flat_map(|op| match op {
                OpCode::NoOp => vec![0].into_iter(),
                OpCode::AddX(val) => vec![0, val].into_iter(),
            })
            .scan(1, |state, offset| {
                *state += offset;
                Some(*state)
            }),
    )
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 1;

    type ParsedInput = Vec<OpCode>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        lines.map(|line| line.parse()).collect()
    }

    fn part_1(
        op_codes: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(register_x_during_cycle(op_codes.iter().copied())
            .enumerate()
            .map(|(i, val)| (i + 1, val))
            // Skip 19, not 20, because .enumerate() counts from 0
            // where the puzzle counts from 1.
            .skip(19)
            .step_by(40)
            .map(|(cycle, x)| (cycle as i32) * x)
            .sum::<i32>())
    }

    fn part_2(
        op_codes: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let res: DisplayString =
            register_x_during_cycle(op_codes.iter().copied())
                .chunks(40)
                .into_iter()
                .map(|chunk| {
                    chunk
                        .enumerate()
                        .map(|(i, value)| ((i as i32) - value).abs() <= 1)
                        .map(|is_lit| if is_lit { '#' } else { '.' })
                        .collect::<String>()
                })
                .join("\n")
                .into();
        Ok(res)
    }
}
