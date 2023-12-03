use aoc_utils::prelude::*;

use std::collections::HashMap;

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

fn num_vowels(string: &str) -> usize {
    string
        .chars()
        .filter(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u'))
        .count()
}

fn has_double_letter(string: &str) -> bool {
    string.chars().tuple_windows().any(|(a, b)| a == b)
}

fn has_naughty_string(string: &str) -> bool {
    string.chars().tuple_windows().any(|(a, b)| {
        matches!((a, b), ('a', 'b') | ('c', 'd') | ('p', 'q') | ('x', 'y'))
    })
}

fn has_repeated_pair(string: &str) -> bool {
    let mut seen: HashMap<(char, char), usize> = HashMap::new();
    for (i, pair) in string.chars().tuple_windows().enumerate() {
        if let Some(min_pos) = seen.get(&pair) {
            if i >= *min_pos {
                return true;
            }
        }
        seen.insert(pair, i + 2);
    }

    false
}

fn has_sandwich(string: &str) -> bool {
    string.chars().tuple_windows().any(|(a, _, c)| a == c)
}

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<String>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        Ok(lines.map(|line| line.to_string()).collect())
    }

    fn part_1(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = values
            .iter()
            .filter(|string| {
                num_vowels(string) >= 3
                    && has_double_letter(string)
                    && !has_naughty_string(string)
            })
            .count();
        Ok(value)
    }

    fn part_2(
        values: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let value = values
            .iter()
            .filter(|s| has_repeated_pair(s) && has_sandwich(s))
            .count();
        Ok(value)
    }
}
