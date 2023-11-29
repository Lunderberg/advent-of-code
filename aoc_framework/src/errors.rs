#[derive(Debug)]
pub enum Error {
    // Used by framework
    NoCachedInputAvailable,
    ExampleBlockNotFound(u8),
    MissingAdventOfCodeSessionId,
    NotYetImplemented,

    WrappedError(Box<dyn std::error::Error>),
    ExpectedExactlyOne,

    // Used by puzzle solutions.
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
    NoStartPosition,
    NoEndPosition,
    GraphHasCycle,
    DotOnFoldLine,
    NoPathToDest,
    UnknownTypeId(u8),
    UnexpectedEndOfStream,
    IllegalNumberOfOperands,
    UnexpectedToken(String),
    NestingLimitExceeded,
    TooManyValues,
    NotEnoughValues,
    InsufficientSharedBeacons,
    NeverFoundMatchedScanner,
    NoAmphipodAtCurrentLocation,
    AmphipodAtTargetLocation,
    TooManyAmphipodsForRoom,
    InsufficientInputValues,

    // 2015-12-07
    CycleDetected,
    MissingValue(String),

    // 2022-12-07
    NoSuchINode(usize),
    NotADirectory(usize),
    AlreadyAtRootDir,
    ParentNotADirectory,
    NameNotFoundInDirectory,

    // 2022-12-21
    NotFullySimplified(String),
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

impl<T> From<T> for Error
where
    T: 'static,
    T: std::error::Error,
{
    fn from(value: T) -> Self {
        Self::WrappedError(Box::new(value))
    }
}

// impl<T> From<itertools::structs::ExactlyOneError<T>> for Error
// where
//     T: Iterator,
// {
//     fn from(_e: itertools::structs::ExactlyOneError<T>) -> Self {
//         Error::TooManyIteratorItems
//     }
// }
