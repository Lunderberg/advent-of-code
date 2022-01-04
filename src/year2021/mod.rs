mod day01;
mod day02;
mod day03;
mod day04;
mod day05;
mod day06;
mod day07;
mod day08;
mod day09;
mod day10;
mod day11;
mod day12;
mod day13;
mod day14;
mod day15;
mod day16;
mod day17;
mod day18;
mod day19;
mod day20;
mod day21;
mod day22;
mod day23;
mod day24;
mod day25;

use crate::framework::{PuzzleRunner, PuzzleRunnerImpl};

pub fn solutions() -> Vec<Box<dyn PuzzleRunner>> {
    vec![
        PuzzleRunnerImpl::<day01::Day01>::new(),
        PuzzleRunnerImpl::<day02::Day02>::new(),
        PuzzleRunnerImpl::<day03::Day03>::new(),
        PuzzleRunnerImpl::<day04::Day04>::new(),
        PuzzleRunnerImpl::<day05::Day05>::new(),
        PuzzleRunnerImpl::<day06::Day06>::new(),
        PuzzleRunnerImpl::<day07::Day07>::new(),
        PuzzleRunnerImpl::<day08::Day08>::new(),
        PuzzleRunnerImpl::<day09::Day09>::new(),
        PuzzleRunnerImpl::<day10::Day10>::new(),
        PuzzleRunnerImpl::<day11::Day11>::new(),
        PuzzleRunnerImpl::<day12::Day12>::new(),
        PuzzleRunnerImpl::<day13::Day13>::new(),
        PuzzleRunnerImpl::<day14::Day14>::new(),
        PuzzleRunnerImpl::<day15::Day15>::new(),
        PuzzleRunnerImpl::<day16::Day16>::new(),
        PuzzleRunnerImpl::<day17::Day17>::new(),
        PuzzleRunnerImpl::<day18::Day18>::new(),
        PuzzleRunnerImpl::<day19::Day19>::new(),
        PuzzleRunnerImpl::<day20::Day20>::new(),
        PuzzleRunnerImpl::<day21::Day21>::new(),
        PuzzleRunnerImpl::<day22::Day22>::new(),
        PuzzleRunnerImpl::<day23::Day23>::new(),
        PuzzleRunnerImpl::<day24::Day24>::new(),
        PuzzleRunnerImpl::<day25::Day25>::new(),
    ]
}
