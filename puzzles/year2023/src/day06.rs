use aoc_utils::prelude::*;

fn concat_num(a: u64, b: u64) -> u64 {
    10u64.pow(b.ilog10() + 1) * a + b
}

fn num_winning_times(race_time: u64, record_dist: u64) -> u64 {
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

    // let a = 1.0;
    // let b = -(race_time as f64);
    // let c = record_dist as f64;

    let disc = race_time * race_time - 4 * record_dist;
    // The only part that needs to be done with f64, at least until
    // u64::isqrt is added in
    // https://github.com/rust-lang/rust/issues/116226.
    let sqrt_disc = (disc as f64).sqrt().floor() as u64;

    // floor((-b + sqrt_disc) / 2 - (-b - sqrt_disc) / 2) + 1
    // floor(-b/2 + sqrt_disc/2 + b/2 + sqrt_disc/2) + 1
    // floor(sqrt_disc) + 1
    sqrt_disc + 1
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
            })
            .product::<u64>();

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

        let value = num_winning_times(race_time, record_dist);

        Ok(value)
    }
}
