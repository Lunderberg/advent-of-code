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

pub fn solutions() -> impl Iterator<Item = Box<dyn PuzzleRunner>> {
    vec![
        PuzzleRunnerImpl::<day01::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day02::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day03::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day04::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day05::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day06::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day07::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day08::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day09::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day10::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day11::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day12::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day13::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day14::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day15::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day16::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day17::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day18::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day19::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day20::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day21::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day22::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day23::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day24::ThisDay>::new_box(),
        PuzzleRunnerImpl::<day25::ThisDay>::new_box(),
    ]
    .into_iter()
}
