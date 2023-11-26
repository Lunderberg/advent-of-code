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

    fn mod_name(&self) -> impl Display + '_ {
        format!("puzzle_{}_{}", self.year_str(), self.day_str())
    }

    fn year_str(&self) -> &str {
        self.0
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
    }

    fn day_str(&self) -> &str {
        self.0.file_stem().unwrap().to_str().unwrap()
    }
}

fn collect_all_solutions(base_dir: &Path) -> Vec<PuzzlePath> {
    let mut output: Vec<_> = base_dir
        .read_dir()
        .unwrap()
        .map(|res| res.unwrap().path())
        .filter(|path| {
            path.file_name()
                .expect("Last segment should not be ..")
                .to_str()
                .expect("Cannot convert path to UTF-8")
                .starts_with("year")
        })
        .flat_map(|year_path| {
            year_path
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
        })
        .map(|path| PuzzlePath(path))
        .collect();

    output.sort();

    output
}

fn generate_solution_iter(
    solutions: &[PuzzlePath],
    out_file: &mut impl std::io::Write,
) -> Result<(), std::io::Error> {
    solutions.iter().try_for_each(|puzzle| {
        write!(out_file, "#[path = \"{}\"]\n", puzzle.path_display())?;
        write!(out_file, "mod {};\n", puzzle.mod_name())
    })?;

    write!(
        out_file,
        "pub fn solutions() -> impl Iterator<Item = Box<dyn ::aoc::framework::PuzzleRunner>> {{\n")?;
    write!(out_file, "    [\n")?;
    solutions
        .iter()
        .try_for_each(|puzzle| {
            write!(
                out_file,
                "        ::aoc::framework::PuzzleRunnerImpl::<{}::ThisDay>::new_box(),\n",
                puzzle.mod_name()
            )
        })?;
    write!(out_file, "    ].into_iter()\n")?;
    write!(out_file, "}}")?;

    Ok(())
}

fn main() {
    let cargo_dir: std::path::PathBuf =
        std::env::var("CARGO_MANIFEST_DIR").unwrap().into();

    let solutions = collect_all_solutions(&cargo_dir.join("src"));

    let out_dir: PathBuf = std::env::var("OUT_DIR").unwrap().into();
    let mut out_file =
        File::create(out_dir.join("collected_solutions.rs")).unwrap();

    generate_solution_iter(&solutions, &mut out_file).unwrap();
}
