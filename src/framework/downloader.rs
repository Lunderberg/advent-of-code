use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use itertools::Itertools;

use crate::Error;

use html5ever::tendril::TendrilSink;
use markup5ever_rcdom::{Handle, NodeData};
use ratelimit::Ratelimiter;

pub struct Downloader {
    aoc_session_id: String,
    rate_limiter: Ratelimiter,
    cache: HashMap<DownloadTarget, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DownloadSource {
    User,
    Example(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DownloadTarget {
    source: DownloadSource,
    year: u32,
    day: u32,
}

impl Downloader {
    pub fn new() -> Result<Downloader, Error> {
        // No more than one interaction every 5 seconds.
        let rate_limiter =
            Ratelimiter::builder(1, std::time::Duration::new(5, 0))
                .build()
                .unwrap();
        let aoc_session_id = std::env::var("AOC_SESSION_ID")
            .map_err(|_| Error::MissingAdventOfCodeSessionId)?;
        Ok(Downloader {
            rate_limiter,
            aoc_session_id,
            cache: HashMap::new(),
        })
    }

    pub fn puzzle_input(
        &mut self,
        year: u32,
        day: u32,
        input_source: DownloadSource,
    ) -> Result<impl Iterator<Item = &str>, Error> {
        let target = DownloadTarget {
            year,
            day,
            source: input_source,
        };
        if !self.cache.contains_key(&target) {
            self.load_to_cache(target)?;
        }
        Ok(self.cache[&target].lines())
    }

    fn load_to_cache(&mut self, target: DownloadTarget) -> Result<(), Error> {
        let input_string = match target.source {
            DownloadSource::User => {
                self.user_puzzle_input(target.year, target.day)
            }
            DownloadSource::Example(example_num) => {
                self.example_puzzle_input(target.year, target.day, example_num)
            }
        }?;
        self.cache.insert(target, input_string);
        Ok(())
    }

    fn user_puzzle_input(
        &mut self,
        year: u32,
        day: u32,
    ) -> Result<String, Error> {
        let url = format!("https://adventofcode.com/{year}/day/{day}/input");
        let filename = self.cache_file_loc(url)?;
        Ok(std::fs::read_to_string(filename)?)
    }

    fn example_puzzle_input(
        &mut self,
        year: u32,
        day: u32,
        example_num: usize,
    ) -> Result<String, Error> {
        let url = format!("https://adventofcode.com/{year}/day/{day}");
        let cache_file = self.cache_file_loc(url)?;
        self.find_example_blocks(cache_file)?
            .nth(example_num)
            .ok_or(Error::NoneError)
    }

    fn find_example_blocks<P>(
        &self,
        path: P,
    ) -> Result<impl Iterator<Item = String>, Error>
    where
        P: AsRef<Path>,
    {
        let opts = html5ever::driver::ParseOpts {
            tree_builder: html5ever::tree_builder::TreeBuilderOpts {
                drop_doctype: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let dom = html5ever::parse_document(
            markup5ever_rcdom::RcDom::default(),
            opts,
        )
        .from_utf8()
        .from_file(path)?;

        Ok(dom
            .document
            .walk()
            .filter(|handle| match handle.data {
                NodeData::Element { ref name, .. } => {
                    name.local.to_string() == "pre"
                }
                _ => false,
            })
            .map(|handle| {
                handle
                    .walk()
                    .filter_map(|handle| match &handle.data {
                        NodeData::Text { contents } => {
                            Some(contents.borrow().to_string())
                        }
                        _ => None,
                    })
                    .join("")
            }))
    }

    fn wait_for_rate_limit(&mut self) {
        while let Err(sleep) = self.rate_limiter.try_wait() {
            std::thread::sleep(sleep);
        }
    }

    fn cache_file_loc<U: reqwest::IntoUrl>(
        &mut self,
        url: U,
    ) -> Result<PathBuf, Error> {
        let path: PathBuf = [
            ".",
            ".cache",
            &self.aoc_session_id,
            &url.as_str().replace('/', "_"),
        ]
        .iter()
        .collect();

        if !path.exists() {
            self.wait_for_rate_limit();
            let client = reqwest::blocking::Client::new();
            let mut response = client
                .get(url)
                .header("cookie", format!("session={}", self.aoc_session_id))
                .send()?;

            std::fs::create_dir_all(path.parent().ok_or(Error::NoneError)?)?;
            let file = File::create(path.clone())?;
            io::copy(&mut response, &mut io::BufWriter::new(file)).or_else(
                |err| {
                    std::fs::remove_file(path.clone())?;
                    Err(err)
                },
            )?;
        }

        Ok(path)
    }
}

trait GraphWalker {
    fn walk(&self) -> RcDomWalker;
}

impl GraphWalker for Handle {
    fn walk(&self) -> RcDomWalker {
        RcDomWalker::new(self.clone())
    }
}

struct RcDomWalker {
    root: Option<Handle>,
    stack: Vec<(Handle, usize)>,
}

impl RcDomWalker {
    fn new(root: Handle) -> Self {
        let stack = Vec::new();
        Self {
            root: Some(root),
            stack,
        }
    }
}

impl Iterator for RcDomWalker {
    type Item = Handle;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(handle) = self.root.take() {
            self.stack.push((handle.clone(), 0));
            return Some(handle);
        }

        while !self.stack.is_empty() {
            let (parent, index) = self.stack.last_mut().unwrap();
            let child = parent.children.borrow().get(*index).cloned();

            if let Some(child) = child {
                *index += 1;
                self.stack.push((child.clone(), 0));
                return Some(child);
            } else {
                self.stack.pop();
            }
        }

        None
    }
}
