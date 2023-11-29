extern crate self as aoc_framework;

pub mod framework;
pub use framework::{Puzzle, YearDay};

include!(concat!(env!("OUT_DIR"), "/collected_solutions.rs"));

mod errors;
pub use errors::Error;

pub mod prelude;
