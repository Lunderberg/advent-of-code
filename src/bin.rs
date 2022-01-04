use structopt::StructOpt;

use aoc::{
    framework::{Downloader, PuzzleInputSource, PuzzlePart, PuzzleRunner},
    Error,
};

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short = "y", long = "year")]
    year: Option<u32>,

    #[structopt(short = "d", long = "day")]
    days: Option<Vec<u8>>,

    #[structopt(short = "e", long = "example-input")]
    use_example_input: bool,
}

fn main() -> Result<(), Error> {
    let runners: Vec<Box<dyn PuzzleRunner>> = aoc::year2021::solutions();

    let opt = Options::from_args();

    let year = opt.year.unwrap_or_else(|| {
        runners
            .iter()
            .filter(|runner| runner.implemented())
            .map(|runner| runner.year())
            .max()
            .unwrap()
    });

    let days: Vec<_> = opt.days.unwrap_or_else(|| {
        vec![runners
            .iter()
            .filter(|runner| runner.implemented() && runner.year() == year)
            .map(|runner| runner.day())
            .max()
            .unwrap()]
    });

    let mut active_runners: Vec<_> = runners
        .into_iter()
        .filter(|runner| {
            runner.implemented()
                && runner.year() == year
                && days.contains(&runner.day())
        })
        .collect();

    let input_source = if opt.use_example_input {
        PuzzleInputSource::Example
    } else {
        PuzzleInputSource::User
    };

    let mut downloader = Downloader::new()?;

    active_runners.iter_mut().try_for_each(|runner| {
        runner.parse_inputs(&mut downloader, input_source)
    })?;

    active_runners
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
