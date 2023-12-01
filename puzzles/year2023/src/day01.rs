use aoc_utils::prelude::*;

use regex::Regex;

fn to_digit(s: &str) -> Option<u32> {
    match s {
        "0" | "zero" => Some(0),
        "1" | "one" => Some(1),
        "2" | "two" => Some(2),
        "3" | "three" => Some(3),
        "4" | "four" => Some(4),
        "5" | "five" => Some(5),
        "6" | "six" => Some(6),
        "7" | "seven" => Some(7),
        "8" | "eight" => Some(8),
        "9" | "nine" => Some(9),
        _ => None,
    }
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<String>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines.map(|line| line.to_string()).collect())
    }

    fn part_1(
        lines: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = lines
            .iter()
            .map(|line| {
                let mut iter = line.chars().filter_map(|c| c.to_digit(10));
                let a = iter.next().unwrap();
                let b = iter.last().unwrap_or(a);
                10 * a + b
            })
            .sum::<u32>();
        Ok(value)
    }

    fn part_2(
        lines: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let reg = Regex::new(
            r"[0-9]|zero|one|two|three|four|five|six|seven|eight|nine",
        )
        .unwrap();

        let value = lines
            .iter()
            .map(|line| {
                // First attempt, doesn't work because
                // Regex::find_iter returns each non-overlapping
                // match.  I want the last match, even if it overlaps
                // with previous words.  (e.g. "zerone" has the last
                // string "one", but it overlaps with "zero".)
                //
                // let mut iter =
                //     reg.find_iter(&line).filter_map(|m| to_digit(m.as_str()));
                // let a = iter.next().unwrap();
                // let b = iter.last().unwrap_or(a);

                let a = to_digit(reg.find(&line).unwrap().as_str()).unwrap();

                let b = (0..line.len())
                    .rev()
                    .find_map(|i| {
                        reg.find_at(&line, i)
                            .map(|m| to_digit(m.as_str()).unwrap())
                    })
                    .unwrap();

                10 * a + b
            })
            .sum::<u32>();
        Ok(value)
    }
}
