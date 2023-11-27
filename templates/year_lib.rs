include!(concat!(
    env!("OUT_DIR"),
    "/collected_solutions_",
    env!("CARGO_PKG_NAME"),
    ".rs",
));

pub(crate) use aoc_framework::utils;
pub(crate) use aoc_framework::{Error, Puzzle};
