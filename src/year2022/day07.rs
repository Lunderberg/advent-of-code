#![allow(unused_imports)]
use crate::{Error, Puzzle};
use std::collections::HashMap;
use std::str::FromStr;

use itertools::Itertools;

#[derive(Debug)]
pub enum TerminalLine {
    ChangeDir(String),
    ListDir,
    Directory(String),
    File(String, usize),
}

impl FromStr for TerminalLine {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "$ ls" {
            Ok(TerminalLine::ListDir)
        } else if let Some(dir) = s.strip_prefix("$ cd ") {
            Ok(TerminalLine::ChangeDir(dir.to_string()))
        } else if let Some(dir) = s.strip_prefix("dir ") {
            Ok(TerminalLine::Directory(dir.to_string()))
        } else if let Some((size, file)) = s.split_once(' ') {
            Ok(TerminalLine::File(file.to_string(), size.parse()?))
        } else {
            Err(Error::InvalidString(s.to_string()))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct INode {
    index: usize,
}

#[derive(Debug)]
struct Directory {
    contents: Vec<INode>,
    parent: INode,
}

#[derive(Debug)]
struct File {
    size: usize,
}

#[derive(Debug)]
enum FileSystemEntryType {
    File(File),
    Directory(Directory),
}

impl From<File> for FileSystemEntryType {
    fn from(file: File) -> Self {
        Self::File(file)
    }
}

impl From<Directory> for FileSystemEntryType {
    fn from(dir: Directory) -> Self {
        Self::Directory(dir)
    }
}

#[derive(Debug)]
struct FileSystemEntry {
    name: String,
    details: FileSystemEntryType,
}

#[derive(Debug)]
pub struct FileSystem {
    entries: Vec<FileSystemEntry>,
}

impl Default for FileSystem {
    fn default() -> Self {
        let root_dir = Directory {
            contents: Vec::new(),
            parent: INode { index: 0 },
        };
        let root = FileSystemEntry {
            name: "/".to_string(),
            details: root_dir.into(),
        };
        Self {
            entries: vec![root],
        }
    }
}

impl FileSystem {
    fn root(&self) -> INode {
        INode { index: 0 }
    }

    fn _get_entry(&self, node: INode) -> Result<&FileSystemEntry, Error> {
        self.entries
            .get(node.index)
            .ok_or_else(|| Error::NoSuchINode(node.index))
    }

    fn _get_dir(&self, node: INode) -> Result<&Directory, Error> {
        match &self._get_entry(node)?.details {
            FileSystemEntryType::Directory(dir) => Ok(dir),
            _ => Err(Error::NotADirectory(node.index)),
        }
    }

    fn _get_mut_entry(
        &mut self,
        node: INode,
    ) -> Result<&mut FileSystemEntry, Error> {
        self.entries
            .get_mut(node.index)
            .ok_or_else(|| Error::NoSuchINode(node.index))
    }

    fn _get_mut_dir(&mut self, node: INode) -> Result<&mut Directory, Error> {
        match &mut self._get_mut_entry(node)?.details {
            FileSystemEntryType::Directory(dir) => Ok(dir),
            _ => Err(Error::NotADirectory(node.index)),
        }
    }

    fn _insert(
        &mut self,
        parent: INode,
        entry: FileSystemEntry,
    ) -> Result<INode, Error> {
        let inode = INode {
            index: self.entries.len(),
        };

        self.entries.push(entry);

        match self._get_mut_dir(parent) {
            Ok(dir) => {
                dir.contents.push(inode);
            }
            Err(err) => {
                self.entries.pop();
                return Err(err);
            }
        }

        Ok(inode)
    }

    fn mkdir(&mut self, parent: INode, name: &str) -> Result<INode, Error> {
        self._insert(
            parent,
            FileSystemEntry {
                name: name.to_string(),
                details: Directory {
                    contents: Vec::new(),
                    parent,
                }
                .into(),
            },
        )
    }

    fn pardir(&self, dir: INode) -> Result<INode, Error> {
        Ok(self._get_dir(dir)?.parent)
    }

    fn find_in_dir(&self, dir: INode, name: &str) -> Result<INode, Error> {
        self._get_dir(dir)?
            .contents
            .iter()
            .copied()
            .find(|inode| {
                self._get_entry(*inode)
                    .ok()
                    .map(|entry| (entry.name == name))
                    .unwrap_or(false)
            })
            .ok_or(Error::NameNotFoundInDirectory)
    }

    fn isdir(&self, dir: INode) -> Result<bool, Error> {
        match self._get_entry(dir)?.details {
            FileSystemEntryType::Directory(_) => Ok(true),
            _ => Ok(false),
        }
    }

    fn subdir(&self, dir: INode, name: &str) -> Result<INode, Error> {
        let sub = self.find_in_dir(dir, name)?;
        if self.isdir(sub)? {
            Ok(sub)
        } else {
            Err(Error::NotADirectory(sub.index))
        }
    }

    fn touch(
        &mut self,
        parent: INode,
        name: &str,
        size: usize,
    ) -> Result<INode, Error> {
        self._insert(
            parent,
            FileSystemEntry {
                name: name.to_string(),
                details: File { size }.into(),
            },
        )
    }

    fn size_recursive(
        &self,
        base_inode: INode,
    ) -> Result<HashMap<INode, usize>, Error> {
        let base_dir = self._get_dir(base_inode)?;

        fn visit_dir(
            file_system: &FileSystem,
            dir_inode: INode,
            dir: &Directory,
            out: &mut HashMap<INode, usize>,
        ) -> Result<(), Error> {
            for content_inode in &dir.contents {
                let entry = file_system._get_entry(*content_inode)?;
                let size: usize = match &entry.details {
                    FileSystemEntryType::File(file) => file.size,
                    FileSystemEntryType::Directory(subdir) => {
                        match out.get(content_inode) {
                            Some(size) => *size,
                            None => {
                                visit_dir(
                                    file_system,
                                    *content_inode,
                                    subdir,
                                    out,
                                )?;
                                out.get(content_inode).copied().unwrap_or(0)
                            }
                        }
                    }
                };
                *out.entry(dir_inode).or_insert(0) += size;
            }

            Ok(())
        }

        let mut out = HashMap::new();

        visit_dir(self, base_inode, base_dir, &mut out)?;

        Ok(out)
    }
}

#[derive(Debug)]
struct SystemState {
    file_system: FileSystem,
    current_dir: INode,
}

impl Default for SystemState {
    fn default() -> Self {
        let file_system: FileSystem = Default::default();
        Self {
            current_dir: file_system.root(),
            file_system,
        }
    }
}

impl SystemState {
    fn apply_line(mut self, line: &TerminalLine) -> Result<Self, Error> {
        match &line {
            TerminalLine::ChangeDir(arg) => match arg.as_str() {
                "/" => {
                    self.current_dir = self.file_system.root();
                }
                ".." => {
                    self.current_dir =
                        self.file_system.pardir(self.current_dir)?;
                }
                _ => {
                    self.current_dir =
                        self.file_system.subdir(self.current_dir, arg)?;
                }
            },
            TerminalLine::ListDir => {}
            TerminalLine::Directory(name) => {
                self.file_system.mkdir(self.current_dir, name)?;
            }
            TerminalLine::File(name, size) => {
                self.file_system.touch(self.current_dir, name, *size)?;
            }
        }
        Ok(self)
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 1;

    type ParsedInput = FileSystem;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let state = lines.map(|s| s.parse::<TerminalLine>()).try_fold(
            SystemState::default(),
            |state: SystemState,
             line: Result<TerminalLine, Error>|
             -> Result<SystemState, Error> {
                state.apply_line(&line?)
            },
        )?;

        Ok(state.file_system)
    }

    fn part_1(
        file_system: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        Ok(file_system
            .size_recursive(file_system.root())?
            .into_values()
            .filter(|dir_size| *dir_size <= 100000)
            .sum::<usize>())
    }

    fn part_2(
        file_system: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let total_space = 70000000;
        let required_space = 30000000;

        let sizes = file_system.size_recursive(file_system.root())?;

        let min_to_delete = sizes.get(&file_system.root()).unwrap()
            - (total_space - required_space);

        Ok(sizes
            .into_values()
            .filter(|dir_size| *dir_size >= min_to_delete)
            .min()
            .unwrap())
    }
}
