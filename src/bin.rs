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
    day: Option<u8>,

    #[structopt(short = "e", long = "example-input")]
    use_example_input: bool,

    #[structopt(short = "v", long = "verbose")]
    verbose: bool,
}

fn main() -> Result<(), Error> {
    let runners: Vec<Box<dyn PuzzleRunner>> = std::iter::empty()
        .chain(aoc::year2021::solutions())
        .chain(aoc::year2022::solutions())
        .collect();

    let opt = Options::from_args();

    let year = opt.year.unwrap_or_else(|| {
        runners.iter().map(|runner| runner.year()).max().unwrap()
    });

    let day = opt.day.unwrap_or_else(|| {
        runners
            .iter()
            .filter(|runner| runner.year() == year)
            .map(|runner| runner.day())
            .max()
            .unwrap()
    });

    let mut runner = runners
        .into_iter()
        .find(|runner| runner.year() == year && runner.day() == day)
        .unwrap();

    let input_source = if opt.use_example_input {
        PuzzleInputSource::Example
    } else {
        PuzzleInputSource::User
    };

    let mut downloader = Downloader::new()?;

    runner.parse_inputs(&mut downloader, input_source, opt.verbose)?;

    PuzzlePart::iter()
        .map(|part| {
            let res = runner.run_puzzle_part(part, input_source);
            (part, res)
        })
        .inspect(|(part, res)| {
            println!("{:04}-12-{:02}, {}", runner.year(), runner.day(), part);
            match res {
                Ok(val) => println!("{val}"),
                Err(error) => println!("Error: {error:?}"),
            }
        })
        .map(|(_part, res)| res)
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(())
}
