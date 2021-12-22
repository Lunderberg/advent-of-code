use crate::utils::{PuzzleRunner, PuzzleRunnerImpl};

use crate::solutions::*;

pub fn iter_solutions() -> impl Iterator<Item = Box<dyn PuzzleRunner>> {
    let puzzles: Vec<Box<dyn PuzzleRunner>> = vec![
        PuzzleRunnerImpl::<Day01>::new(),
        PuzzleRunnerImpl::<Day02>::new(),
        PuzzleRunnerImpl::<Day03>::new(),
        PuzzleRunnerImpl::<Day04>::new(),
        PuzzleRunnerImpl::<Day05>::new(),
        PuzzleRunnerImpl::<Day06>::new(),
        PuzzleRunnerImpl::<Day07>::new(),
        PuzzleRunnerImpl::<Day08>::new(),
        PuzzleRunnerImpl::<Day09>::new(),
        PuzzleRunnerImpl::<Day10>::new(),
        PuzzleRunnerImpl::<Day11>::new(),
        PuzzleRunnerImpl::<Day12>::new(),
        PuzzleRunnerImpl::<Day13>::new(),
        PuzzleRunnerImpl::<Day14>::new(),
        PuzzleRunnerImpl::<Day15>::new(),
        PuzzleRunnerImpl::<Day16>::new(),
        PuzzleRunnerImpl::<Day17>::new(),
        PuzzleRunnerImpl::<Day18>::new(),
        PuzzleRunnerImpl::<Day19>::new(),
        PuzzleRunnerImpl::<Day20>::new(),
        PuzzleRunnerImpl::<Day21>::new(),
        PuzzleRunnerImpl::<Day22>::new(),
        PuzzleRunnerImpl::<Day23>::new(),
        PuzzleRunnerImpl::<Day24>::new(),
        PuzzleRunnerImpl::<Day25>::new(),
    ];

    puzzles.into_iter().filter(|p| p.implemented())
}
