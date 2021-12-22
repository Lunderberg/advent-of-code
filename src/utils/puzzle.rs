use crate::utils::Error;

use std::collections::HashMap;
use std::env;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};

const YEAR: i32 = 2021;

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
    fn day(&self) -> u8;
    fn implemented(&self) -> bool;

    // Download and parse the results
    fn parse_inputs(
        &mut self,
        //downloader: &Downloader,
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
    fn day(&self) -> u8 {
        T::DAY
    }

    fn implemented(&self) -> bool {
        T::IMPLEMENTED
    }

    fn parse_inputs(
        &mut self,
        //downloader: &Downloader,
        input_source: PuzzleInputSource,
    ) -> Result<(), Error> {
        let text = match input_source {
            PuzzleInputSource::User => user_puzzle_input(YEAR, T::DAY)?,
            PuzzleInputSource::Example => {
                example_puzzle_input(YEAR, T::DAY, T::EXAMPLE_NUM)?
            }
        };
        let parsed_input = T::parse_input(text.lines())?;

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

pub enum PuzzleInput {
    User,
    Example(i32),
}

pub trait PuzzleExtensions {
    fn puzzle_input(&self, which_input: PuzzleInput) -> Result<String, Error>;
}

impl<T> PuzzleExtensions for T
where
    T: Puzzle,
{
    fn puzzle_input(&self, which_input: PuzzleInput) -> Result<String, Error> {
        match which_input {
            PuzzleInput::User => self.user_puzzle_input(),
            PuzzleInput::Example(i) => self.example_puzzle_input(i),
        }
    }
}

static mut DOWNLOAD_RATE_LIMITER: Option<ratelimit::Limiter> = None;

fn wait_rate_limit() {
    // Unsafe in the case of multithreading, which I don't intend to
    // do.  I probably should come up with a better method in the
    // future.
    unsafe {
        if DOWNLOAD_RATE_LIMITER.is_none() {
            DOWNLOAD_RATE_LIMITER = Some(
                ratelimit::Builder::new()
                    .capacity(1)
                    .quantum(1)
                    .interval(std::time::Duration::new(5, 0))
                    .build(),
            );
        }

        DOWNLOAD_RATE_LIMITER.as_mut().unwrap().wait();
    }
}

trait PrivatePuzzleExtensions {
    fn user_puzzle_input(&self) -> Result<String, Error>;

    fn example_puzzle_input(&self, example_num: i32) -> Result<String, Error>;

    fn download_url<T: reqwest::IntoUrl>(
        &self,
        url: T,
    ) -> Result<String, Error>;

    fn find_example_blocks(&self, html: &str) -> Result<Vec<String>, Error>;
}

impl<T> PrivatePuzzleExtensions for T
where
    T: Puzzle,
{
    fn user_puzzle_input(&self) -> Result<String, Error> {
        self.download_url(format!(
            "https://adventofcode.com/{}/day/{}/input",
            YEAR,
            T::DAY
        ))
    }

    fn find_example_blocks(&self, html: &str) -> Result<Vec<String>, Error> {
        let opts = html5ever::driver::ParseOpts {
            tree_builder: html5ever::tree_builder::TreeBuilderOpts {
                drop_doctype: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let dom = parse_document(RcDom::default(), opts)
            .from_utf8()
            .read_from(&mut html.as_bytes())?;

        fn unpack_block(handle: &Handle) -> String {
            if let NodeData::Text { contents } = &handle.data {
                contents.borrow().to_string()
            } else {
                handle
                    .children
                    .borrow()
                    .iter()
                    .map(|child| unpack_block(child))
                    .collect()
            }
        }

        fn search_for_block(handle: &Handle, output: &mut Vec<String>) {
            if let NodeData::Element { ref name, .. } = handle.data {
                if name.local.to_string() == "pre" {
                    output.push(unpack_block(handle));
                }
            }
            handle
                .children
                .borrow()
                .iter()
                .for_each(|child| search_for_block(child, output));
        }

        let mut output = Vec::new();
        search_for_block(&dom.document, &mut output);

        Ok(output)
    }

    fn example_puzzle_input(&self, example_num: i32) -> Result<String, Error> {
        let html = self.download_url(format!(
            "https://adventofcode.com/{}/day/{}",
            YEAR,
            T::DAY
        ))?;

        Ok(self
            .find_example_blocks(&html)?
            .get(example_num as usize)
            .ok_or(Error::InvalidArg(crate::utils::Arg::I32(example_num)))?
            .to_string())
    }

    fn download_url<U: reqwest::IntoUrl>(
        &self,
        url: U,
    ) -> Result<String, Error> {
        let session_id = env::var("AOC_SESSION_ID")?;

        let path: PathBuf =
            [".", ".cache", &session_id, &url.as_str().replace("/", "_")]
                .iter()
                .collect::<PathBuf>();

        if path.exists() {
            Ok(std::fs::read_to_string(path)?)
        } else {
            wait_rate_limit();
            let client = reqwest::blocking::Client::new();
            let response = client
                .get(url)
                .header("cookie", format!("session={}", session_id))
                .send()?
                .text()?;

            std::fs::create_dir_all(path.parent().ok_or(Error::NoneError)?)?;
            std::fs::write(path, &response)?;

            Ok(response)
        }
    }
}

fn user_puzzle_input(year: i32, day: u8) -> Result<String, Error> {
    download_url(format!(
        "https://adventofcode.com/{}/day/{}/input",
        year, day
    ))
}

fn find_example_blocks(html: &str) -> Result<Vec<String>, Error> {
    let opts = html5ever::driver::ParseOpts {
        tree_builder: html5ever::tree_builder::TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let dom = parse_document(RcDom::default(), opts)
        .from_utf8()
        .read_from(&mut html.as_bytes())?;

    fn unpack_block(handle: &Handle) -> String {
        if let NodeData::Text { contents } = &handle.data {
            contents.borrow().to_string()
        } else {
            handle
                .children
                .borrow()
                .iter()
                .map(|child| unpack_block(child))
                .collect()
        }
    }

    fn search_for_block(handle: &Handle, output: &mut Vec<String>) {
        if let NodeData::Element { ref name, .. } = handle.data {
            if name.local.to_string() == "pre" {
                output.push(unpack_block(handle));
            }
        }
        handle
            .children
            .borrow()
            .iter()
            .for_each(|child| search_for_block(child, output));
    }

    let mut output = Vec::new();
    search_for_block(&dom.document, &mut output);

    Ok(output)
}

fn example_puzzle_input(
    year: i32,
    day: u8,
    example_num: u8,
) -> Result<String, Error> {
    let html =
        download_url(format!("https://adventofcode.com/{}/day/{}", year, day))?;

    Ok(find_example_blocks(&html)?
        .get(example_num as usize)
        .ok_or(Error::ExampleBlockNotFound(example_num))?
        .to_string())
}

fn download_url<U: reqwest::IntoUrl>(url: U) -> Result<String, Error> {
    let session_id = env::var("AOC_SESSION_ID")?;

    let path: PathBuf =
        [".", ".cache", &session_id, &url.as_str().replace("/", "_")]
            .iter()
            .collect::<PathBuf>();

    if path.exists() {
        Ok(std::fs::read_to_string(path)?)
    } else {
        wait_rate_limit();
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(url)
            .header("cookie", format!("session={}", session_id))
            .send()?
            .text()?;

        std::fs::create_dir_all(path.parent().ok_or(Error::NoneError)?)?;
        std::fs::write(path, &response)?;

        Ok(response)
    }
}
