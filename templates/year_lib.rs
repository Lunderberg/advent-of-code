include!(concat!(
    env!("OUT_DIR"),
    "/collected_solutions_",
    env!("CARGO_PKG_NAME"),
    ".rs",
));

#[allow(unused_imports)]
pub(crate) use aoc_utils as utils;
