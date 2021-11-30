#[derive(Debug)]
pub enum Error {
    WrongInt(std::num::ParseIntError),
    IoError(std::io::Error),
    MissingRegex,
    InvalidValue(String),
    NoneError,
    EarlyFailure,
    UnknownChar(char),
    ParseError,
    Mismatch,
    GameFinished,
    GameNotFinished,
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
// impl From<core::option::NoneError> for Error {
//     fn from(e: core::option::NoneError) -> Self {
//         Error::NoneError
//     }
// }
