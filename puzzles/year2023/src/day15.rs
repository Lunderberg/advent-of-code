use std::fmt::Display;

use aoc_utils::prelude::*;

trait HashExt {
    fn aoc_hash(self) -> u8;
}
impl HashExt for &str {
    fn aoc_hash(self) -> u8 {
        self.chars()
            .map(|c| c as u8)
            .fold(0u8, |prev, a| prev.wrapping_add(a).wrapping_mul(17))
    }
}

#[derive(Debug, Clone)]
struct Lens<'a> {
    name: &'a str,
    value: u8,
}

#[derive(Debug)]
enum Command<'a> {
    Set(Lens<'a>),
    Erase(&'a str),
}

#[derive(Debug)]
struct LensBox<'a> {
    contents: Vec<Lens<'a>>,
}

#[derive(Debug)]
struct FocusingSequence<'a> {
    boxes: [LensBox<'a>; 256],
}

impl Command<'_> {
    fn name(&self) -> &str {
        match &self {
            Command::Set(Lens { name, .. }) => name,
            Command::Erase(name) => name,
        }
    }
}

impl<'lens> FocusingSequence<'lens> {
    fn new() -> Self {
        let boxes = [(); 256].map(|_| LensBox::new());
        Self { boxes }
    }

    fn apply(&mut self, command: Command<'lens>) {
        let box_id = command.name().aoc_hash() as usize;
        self.boxes[box_id].apply(command);
    }

    fn focusing_power(&self) -> u64 {
        self.boxes
            .iter()
            .map(|lens_box| lens_box.focusing_power())
            .enumerate()
            .map(|(i, power)| (i as u64 + 1) * power)
            .sum()
    }
}

impl Display for FocusingSequence<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.boxes
            .iter()
            .enumerate()
            .filter(|(_, lens_box)| !lens_box.contents.is_empty())
            .try_for_each(|(i, lens_box)| writeln!(f, "Box {i}: {lens_box}"))
    }
}

impl<'lens> LensBox<'lens> {
    fn new() -> Self {
        Self {
            contents: Vec::new(),
        }
    }

    fn apply(&mut self, command: Command<'lens>) {
        match command {
            Command::Set(new_lens) => {
                if let Some(to_update) = self
                    .contents
                    .iter_mut()
                    .find(|lens| lens.name == new_lens.name)
                {
                    to_update.value = new_lens.value;
                } else {
                    self.contents.push(new_lens);
                }
            }
            Command::Erase(name) => {
                self.contents.retain(|lens| lens.name != name);
            }
        }
    }

    fn focusing_power(&self) -> u64 {
        self.contents
            .iter()
            .map(|lens| lens.value as u64)
            .enumerate()
            .map(|(i, power)| (i as u64 + 1) * power)
            .sum::<u64>()
    }
}

impl Display for LensBox<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        self.contents.iter().enumerate().try_for_each(|(i, lens)| {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{lens}")
        })?;
        write!(f, "]")
    }
}

impl Display for Lens<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Lens { name, value } = self;
        write!(f, "{name} {value}")
    }
}

impl<'a> TryFrom<&'a str> for Command<'a> {
    type Error = Error;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        let (name, value) = s
            .split(|c| matches!(c, '-' | '='))
            .collect_tuple()
            .ok_or(Error::WrongIteratorSize)?;

        let command = if value.is_empty() {
            Command::Erase(name)
        } else {
            let value = value.parse()?;
            Command::Set(Lens { name, value })
        };
        Ok(command)
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = String;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines
            .exactly_one()
            .map_err(|_| Error::WrongIteratorSize)?
            .to_string())
    }

    fn part_1(
        sequence: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = sequence
            .split(',')
            .map(|item| item.aoc_hash())
            .map(|v| v as u32)
            .sum::<u32>();
        Ok(value)
    }

    fn part_2(
        sequence: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let state = sequence
            .split(',')
            .map(|command| -> Command {
                command.try_into().expect("Failed to parse Lens")
            })
            .fold(FocusingSequence::new(), |mut state, command| {
                state.apply(command);
                state
            });

        Ok(state.focusing_power())
    }
}
