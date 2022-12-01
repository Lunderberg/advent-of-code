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
        PuzzleRunnerImpl::<day01::Day01>::new_box(),
        PuzzleRunnerImpl::<day02::Day02>::new_box(),
        PuzzleRunnerImpl::<day03::Day03>::new_box(),
        PuzzleRunnerImpl::<day04::Day04>::new_box(),
        PuzzleRunnerImpl::<day05::Day05>::new_box(),
        PuzzleRunnerImpl::<day06::Day06>::new_box(),
        PuzzleRunnerImpl::<day07::Day07>::new_box(),
        PuzzleRunnerImpl::<day08::Day08>::new_box(),
        PuzzleRunnerImpl::<day09::Day09>::new_box(),
        PuzzleRunnerImpl::<day10::Day10>::new_box(),
        PuzzleRunnerImpl::<day11::Day11>::new_box(),
        PuzzleRunnerImpl::<day12::Day12>::new_box(),
        PuzzleRunnerImpl::<day13::Day13>::new_box(),
        PuzzleRunnerImpl::<day14::Day14>::new_box(),
        PuzzleRunnerImpl::<day15::Day15>::new_box(),
        PuzzleRunnerImpl::<day16::Day16>::new_box(),
        PuzzleRunnerImpl::<day17::Day17>::new_box(),
        PuzzleRunnerImpl::<day18::Day18>::new_box(),
        PuzzleRunnerImpl::<day19::Day19>::new_box(),
        PuzzleRunnerImpl::<day20::Day20>::new_box(),
        PuzzleRunnerImpl::<day21::Day21>::new_box(),
        PuzzleRunnerImpl::<day22::Day22>::new_box(),
        PuzzleRunnerImpl::<day23::Day23>::new_box(),
        PuzzleRunnerImpl::<day24::Day24>::new_box(),
        PuzzleRunnerImpl::<day25::Day25>::new_box(),
    ]
}
