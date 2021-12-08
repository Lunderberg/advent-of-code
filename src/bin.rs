use itertools::Itertools;
use structopt::StructOpt;

use aoc2021::utils;
use aoc2021::utils::Error;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short = "d", long = "days")]
    days: Option<Vec<i32>>,
}

fn main() -> Result<(), Error> {
    let opt = Options::from_args();

    let days: Vec<i32> = opt.days.unwrap_or_else(|| {
        utils::iter_solutions()
            .last()
            .iter()
            .map(|p| p.day())
            .collect::<Vec<i32>>()
    });

    days.iter()
        .cloned()
        .cartesian_product(vec![1, 2])
        .map(|(day, part)| -> (i32,i32,Result<Box<dyn std::fmt::Debug>, Error>) {
            let res = utils::iter_solutions()
                .filter(|p| p.day() == day)
                .next()
                .ok_or(Error::NoneError)
                .and_then(|puzzle| puzzle.call_part(part));
            (day, part, res)
        })
        .inspect(|(day, part, res)| {
            println!("Day {:02}, Part {:}", day, part);
            println!("{:?}", res);
        } ).map(|(_day,_part,res)| res)
        .collect::<Result<Vec<_>,Error>>()?;

    Ok(())
}
