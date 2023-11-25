extern crate self as aoc;

pub mod framework;
pub use framework::{Puzzle, YearDay};

pub mod utils;
pub mod year2021;
pub mod year2022;

mod errors;
pub use errors::Error;
