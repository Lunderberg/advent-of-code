mod downloader;
pub use downloader::{DownloadSource, Downloader};

mod puzzle;
pub use puzzle::{
    Puzzle, PuzzleInputSource, PuzzlePart, PuzzleRunner, PuzzleRunnerImpl,
    YearDay,
};
