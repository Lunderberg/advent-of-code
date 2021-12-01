#[derive(Debug)]
pub enum Error {
    WrongInt(std::num::ParseIntError),
    IoError(std::io::Error),
    InvalidArg(Arg),
    EnvError(std::env::VarError),
    HttpError(reqwest::Error),
    MissingRegex,
    NoneError,
    EarlyFailure,
    UnknownChar(char),
    ParseError,
    Mismatch,
}

#[derive(Debug)]
pub enum Arg {
    String(String),
    I32(i32),
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
