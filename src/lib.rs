extern crate self as aoc;

pub mod framework;
pub use framework::{Puzzle, YearDay};

pub mod utils;

include!(concat!(env!("OUT_DIR"), "/collected_solutions.rs"));

mod errors;
pub use errors::Error;
