use structopt::StructOpt;

use aoc2021::utils;
use aoc2021::utils::{Error, PuzzleInputSource, PuzzlePart, PuzzleRunner};

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short = "d", long = "days")]
    days: Option<Vec<u8>>,

    #[structopt(short = "e", long = "example-input")]
    use_example_input: bool,
}

fn main() -> Result<(), Error> {
    let opt = Options::from_args();

    let days: Vec<_> = opt.days.unwrap_or_else(|| {
        utils::iter_solutions()
            .last()
            .iter()
            .map(|p| p.day())
            .collect()
    });

    let input_source = if opt.use_example_input {
        PuzzleInputSource::Example
    } else {
        PuzzleInputSource::User
    };

    let mut downloader = utils::Downloader::new()?;

    let mut runners: Vec<Box<dyn PuzzleRunner>> = days
        .into_iter()
        .map(|day| {
            utils::iter_solutions()
                .filter(|p| p.day() == day)
                .next()
                .ok_or(Error::NoneError)
        })
        .collect::<Result<Vec<_>, _>>()?;

    runners.iter_mut().try_for_each(|runner| {
        runner.parse_inputs(&mut downloader, input_source)
    })?;

    runners
        .iter()
        .flat_map(|runner| PuzzlePart::iter().map(move |part| (runner, part)))
        .map(|(runner, puzzle_part)| {
            let res = runner.run_puzzle_part(puzzle_part, input_source);
            (runner.day(), puzzle_part, res)
        })
        .inspect(|(day, part, res)| {
            println!("Day {:02}, {}", day, part);
            match res {
                Ok(val) => println!("{}", val),
                Err(error) => println!("Error: {:?}", error),
            }
        })
        .map(|(_day, _part, res)| res)
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(())
}
