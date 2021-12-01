use crate::utils::Error;

use std::env;
use std::path::PathBuf;

use html5ever::parse_document;
use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData, RcDom};

const YEAR: i32 = 2021;

pub trait Puzzle {
    fn day(&self) -> i32;
    fn implemented(&self) -> bool;
    fn part_1(&self) -> Result<Box<dyn std::fmt::Debug>, Error>;
    fn part_2(&self) -> Result<Box<dyn std::fmt::Debug>, Error>;
}

impl dyn Puzzle {
    pub fn call_part(
        &self,
        part_num: i32,
    ) -> Result<Box<dyn std::fmt::Debug>, Error> {
        if part_num == 1 {
            self.part_1()
        } else if part_num == 2 {
            self.part_2()
        } else {
            Err(Error::InvalidArg(crate::utils::Arg::I32(part_num)))
        }
    }
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
            self.day()
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
            self.day()
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
