#[derive(Debug)]
pub enum Error {
    WrongInt(std::num::ParseIntError),
    IoError(std::io::Error),
    InvalidArg(Arg),
    InvalidIndex(usize),
    EnvError(std::env::VarError),
    HttpError(reqwest::Error),
    MissingRegex,
    NoneError,
    EarlyFailure,
    UnknownChar(char),
    ParseError,
    Mismatch,
    TooManyIteratorItems,
    WrongBingoBoardSize(usize),
    BoardNeverWins,
    NoWinningBoard,
    DiagonalVentLine,
    CannotFindMinMax,
    InvalidDigit(u8),
    NotOpeningDelimiter,
    InvalidString(String),
    NoIllegalCharacterFound,
    NoStartCave,
    NoEndCave,
    GraphHasCycle,
    DotOnFoldLine,
    NoPathToDest,
    UnknownTypeId(u8),
    UnexpectedEndOfStream,
    IllegalNumberOfOperands,
    UnexpectedToken(String),
    NestingLimitExceeded,
}

#[derive(Debug)]
pub enum Arg {
    String(String),
    I32(i32),
}

impl From<&str> for Arg {
    fn from(e: &str) -> Self {
        Arg::String(e.to_string())
    }
}

impl From<i32> for Arg {
    fn from(e: i32) -> Self {
        Arg::I32(e)
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::HttpError(e)
    }
}

impl From<std::env::VarError> for Error {
    fn from(e: std::env::VarError) -> Self {
        Error::EnvError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::WrongInt(e)
    }
}

// Once the try_trait has been stabilized
// impl From<std::option::NoneError> for Error {
//     fn from(e: core::option::NoneError) -> Self {
//         Error::NoneError
//     }
// }

impl<T> From<itertools::structs::ExactlyOneError<T>> for Error
where
    T: Iterator,
{
    fn from(_e: itertools::structs::ExactlyOneError<T>) -> Self {
        Error::TooManyIteratorItems
    }
}
