use aoc_utils::prelude::*;

fn boat_distance(charge_time: u64, race_time: u64) -> u64 {
    race_time.saturating_sub(charge_time) * charge_time
}

fn concat_num(a: u64, b: u64) -> u64 {
    10u64.pow(b.ilog10() + 1) * a + b
}

fn num_winning_times(race_time: u64, record_dist: u64) -> u64 {
    (0..race_time)
        .filter(|&charge_time| {
            boat_distance(charge_time, race_time) > record_dist
        })
        .count() as u64
}

#[derive(aoc_macros::YearDay)]
pub struct ThisDay;

impl Puzzle for ThisDay {
    const EXAMPLE_NUM: u8 = 0;

    type ParsedInput = Vec<(u64, u64)>;
    fn parse_input<'a>(
        lines: impl Iterator<Item = &'a str>,
    ) -> Result<Self::ParsedInput, Error> {
        let (time, dist) = lines
            .map(|line| {
                line.split_ascii_whitespace()
                    .skip(1)
                    .map(|item| item.parse())
            })
            .collect_tuple()
            .unwrap();
        time.zip(dist)
            .map(|(a, b)| Ok((a?, b?)))
            .collect::<Result<Vec<_>, Error>>()
    }

    fn part_1(
        races: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        println!("Races: {races:?}");

        let value = races
            .iter()
            .map(|&(race_time, record_dist)| {
                num_winning_times(race_time, record_dist)
            })
            .fold(1u64, |a, b| a * b);

        Ok(value)
    }

    fn part_2(
        races: &Self::ParsedInput,
    ) -> Result<impl std::fmt::Debug, Error> {
        let (race_time, record_dist) = races.iter().fold(
            (0u64, 0u64),
            |(a_time, a_dist), &(b_time, b_dist)| {
                (concat_num(a_time, b_time), concat_num(a_dist, b_dist))
            },
        );

        println!("Time: {race_time}");
        println!("Dist: {record_dist}");

        let value = num_winning_times(race_time, record_dist);

        Ok(value)
    }
}
