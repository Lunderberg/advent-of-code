use super::{DownloadSource, Downloader};
use crate::Error;

use std::collections::HashMap;
use std::fmt::{Display, Formatter};

const YEAR: u32 = 2021;

#[derive(Debug, Clone, Copy)]
pub enum PuzzlePart {
    Part1,
    Part2,
}

impl PuzzlePart {
    pub fn iter() -> impl Iterator<Item = PuzzlePart> {
        use PuzzlePart::*;
        vec![Part1, Part2].into_iter()
    }

    pub fn part_num(&self) -> u8 {
        use PuzzlePart::*;
        match self {
            Part1 => 1,
            Part2 => 2,
        }
    }
}

impl Display for PuzzlePart {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Part {}", self.part_num())
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum PuzzleInputSource {
    User,
    Example,
}

pub trait PuzzleRunner {
    fn year(&self) -> u32;
    fn day(&self) -> u8;
    fn implemented(&self) -> bool;

    // Download and parse the results
    fn parse_inputs(
        &mut self,
        downloader: &mut Downloader,
        input_source: PuzzleInputSource,
    ) -> Result<(), Error>;

    // Run the puzzle, using the cached inputs.  If successful, return
    // the string output from the puzzle.  If unsuccessful, or if
    // parse_inputs() hasn't been called for that input source, should
    // return an error.
    fn run_puzzle_part(
        &self,
        puzzle_part: PuzzlePart,
        input_source: PuzzleInputSource,
    ) -> Result<String, Error>;
}

pub struct PuzzleRunnerImpl<T: Puzzle> {
    input_cache: HashMap<PuzzleInputSource, T::ParsedInput>,
}

impl<T: 'static> PuzzleRunnerImpl<T>
where
    T: Puzzle,
{
    pub fn new() -> Box<dyn PuzzleRunner> {
        Box::new(Self {
            input_cache: HashMap::new(),
        })
    }
}

impl<T> PuzzleRunner for PuzzleRunnerImpl<T>
where
    T: Puzzle,
{
    fn year(&self) -> u32 {
        T::YEAR
    }

    fn day(&self) -> u8 {
        T::DAY
    }

    fn implemented(&self) -> bool {
        T::IMPLEMENTED
    }

    fn parse_inputs(
        &mut self,
        downloader: &mut Downloader,
        input_source: PuzzleInputSource,
    ) -> Result<(), Error> {
        let download_source = match input_source {
            PuzzleInputSource::User => DownloadSource::User,
            PuzzleInputSource::Example => {
                DownloadSource::Example(T::EXAMPLE_NUM as usize)
            }
        };
        let line_iter =
            downloader.puzzle_input(YEAR, T::DAY as u32, download_source)?;
        let parsed_input = T::parse_input(line_iter)?;

        self.input_cache.insert(input_source, parsed_input);

        Ok(())
    }

    fn run_puzzle_part(
        &self,
        puzzle_part: PuzzlePart,
        input_source: PuzzleInputSource,
    ) -> Result<String, Error> {
        let input = self
            .input_cache
            .get(&input_source)
            .ok_or(Error::NoCachedInputAvailable)?;

        Ok(match puzzle_part {
            PuzzlePart::Part1 => format!("{:?}", T::part_1(&input)?),
            PuzzlePart::Part2 => format!("{:?}", T::part_2(&input)?),
        })
    }
}

pub trait Puzzle {
    const YEAR: u32;
    const DAY: u8;
    const IMPLEMENTED: bool;
    const EXAMPLE_NUM: u8;

    type ParsedInput;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error>;

    type Part1Result: std::fmt::Debug;
    fn part_1(parsed: &Self::ParsedInput) -> Result<Self::Part1Result, Error>;

    type Part2Result: std::fmt::Debug;
    fn part_2(parsed: &Self::ParsedInput) -> Result<Self::Part2Result, Error>;
}
