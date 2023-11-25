extern crate self as aoc;

pub mod framework;
pub use framework::{Puzzle, YearDay};

pub mod utils;

aoc_macros::collect_all_solutions! {}

mod errors;
pub use errors::Error;
