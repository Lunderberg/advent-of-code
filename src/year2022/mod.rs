mod day01;

use crate::framework::{PuzzleRunner, PuzzleRunnerImpl};

pub fn solutions() -> impl Iterator<Item = Box<dyn PuzzleRunner>> {
    vec![PuzzleRunnerImpl::<day01::ThisPuzzle>::new_box()].into_iter()
}
