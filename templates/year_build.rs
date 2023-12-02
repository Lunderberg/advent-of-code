use std::{
    fmt::Display,
    fs::File,
    path::{Path, PathBuf},
};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct PuzzlePath(PathBuf);

impl PuzzlePath {
    fn path_display(&self) -> impl Display + '_ {
        self.0.display()
    }

    fn try_mod_name(&self) -> Option<impl Display> {
        Some(format!("puzzle_{}_{}", self.year_str()?, self.day_str()?))
    }

    fn mod_name(&self) -> impl Display {
        self.try_mod_name().unwrap()
    }

    fn year_str(&self) -> Option<&str> {
        self.0
            .iter()
            .rev()
            .map(|component| component.to_str().unwrap())
            .find(|component| component.starts_with("year"))
    }

    fn day_str(&self) -> Option<&str> {
        self.0
            .iter()
            .rev()
            .map(|component| component.to_str().unwrap())
            .filter(|component| component.starts_with("day"))
            .map(|component| component.trim_end_matches(".rs"))
            .next()
    }
}

fn collect_all_solutions(base_dir: &Path) -> Vec<PuzzlePath> {
    let mut output: Vec<_> = base_dir
        .read_dir()
        .unwrap()
        .map(|res| res.unwrap().path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .expect("Last segment should not be ..")
                    .to_str()
                    .expect("Cannot convert path to UTF-8")
                    .starts_with("day")
        })
        .map(PuzzlePath)
        .collect();

    output.sort();

    output
}

fn generate_solution_iter(
    solutions: &[PuzzlePath],
    out_file: &mut impl std::io::Write,
) -> Result<(), std::io::Error> {
    writeln!(out_file, "// Found {} puzzle solutions", solutions.len())?;
    writeln!(
        out_file,
        "// CARGO_MANIFEST_DIR: {}",
        std::env::var("CARGO_MANIFEST_DIR").unwrap()
    )?;
    writeln!(
        out_file,
        "// CARGO_PKG_NAME: {}",
        std::env::var("CARGO_PKG_NAME").unwrap()
    )?;

    assert!(!solutions.is_empty());

    solutions.iter().try_for_each(|puzzle| {
        writeln!(out_file, "#[path = \"{}\"]", puzzle.path_display())?;
        writeln!(out_file, "mod {};", puzzle.mod_name())
    })?;

    writeln!(
        out_file,
        "pub fn solutions() -> impl Iterator<Item = Box<dyn ::aoc_framework::framework::PuzzleRunner>> {{")?;
    writeln!(out_file, "    [")?;
    solutions
        .iter()
        .try_for_each(|puzzle| {
            writeln!(
                out_file,
                "        ::aoc_framework::framework::PuzzleRunnerImpl::<{}::ThisDay>::new_box(),",
                puzzle.mod_name()
            )
        })?;
    writeln!(out_file, "    ].into_iter()")?;
    write!(out_file, "}}")?;

    Ok(())
}

fn main() {
    let cargo_dir: std::path::PathBuf =
        std::env::var("CARGO_MANIFEST_DIR").unwrap().into();

    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();

    let solutions = collect_all_solutions(&cargo_dir.join("src"));

    let out_dir: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    let mut out_file = File::create(
        out_dir.join(format!("collected_solutions_{crate_name}.rs")),
    )
    .unwrap();

    generate_solution_iter(&solutions, &mut out_file).unwrap();
}
