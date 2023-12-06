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

fn num_winning_times_quadratic(race_time: u64, record_dist: u64) -> u64 {
    // boat_distance(charge_time, race_time) > record_dist
    // record_dist - boat_distance(charge_time, race_time) == 0
    // record_dist - (race_time-charge_time)*charge_time == 0
    // 0 == charge_time**2 - race_time*charge_time + record_dist
    //
    // 0 = a*t^2 + b*t + c
    //
    // a = +1
    // b = -race_time
    // c = record_dist

    let a = 1.0;
    let b = -(race_time as f64);
    let c = record_dist as f64;

    let disc = (b * b - 4.0 * a * c).sqrt();
    let t0 = (-b - disc) / (2.0 * a);
    let t1 = (-b + disc) / (2.0 * a);

    let t0_u64 = t0.ceil() as u64;
    let t1_u64 = t1.floor() as u64;

    let result = t1_u64 - t0_u64 + 1;

    // let known_result = num_winning_times(race_time, record_dist);
    // assert!(
    //     result == known_result,
    //     "For race time = {race_time} and record_dist = {record_dist}, \
    //      expected {known_result}, \
    //      but produced {result} \
    //      with t0 = {t0}, t1 = {t1} \
    //      and t0_u64 = {t0_u64}, t1_u64 = {t1_u64}."
    // );

    result
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
        let value = races
            .iter()
            .map(|&(race_time, record_dist)| {
                num_winning_times(race_time, record_dist)
                // num_winning_times_quadratic(race_time, record_dist)
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

        // let value = num_winning_times(race_time, record_dist);
        let value = num_winning_times_quadratic(race_time, record_dist);

        Ok(value)
    }
}
