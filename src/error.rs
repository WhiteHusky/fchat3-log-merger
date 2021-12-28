use std::io;

use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum Error {
    OutputExists(PathBuf),
    NotEnoughInputs,
    InputDoesNotExist(PathBuf),
    InputIsNotDirectory(PathBuf),
    BadTimeDiff(humantime::DurationError),
    UnableToCreateDirectory(io::Error),
    MessageParseError(fchat3_log_lib::error::Error),
    UnableToOpenIndex(io::Error),
    UnableToOpenFile(io::Error),
    ExitingWithError
}

impl From<fchat3_log_lib::error::Error> for Error {
    fn from(e: fchat3_log_lib::error::Error) -> Self {
        Self::MessageParseError(e)
    }
}

impl From<humantime::DurationError> for Error {
    fn from(e: humantime::DurationError) -> Self {
        Self::BadTimeDiff(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::OutputExists(p) => write!(f, "Output folder already exists: {}", p.to_string_lossy()),
            Error::NotEnoughInputs => write!(f, "Specify more than one input folder."),
            Error::InputDoesNotExist(p) => write!(f, "Input folder does not exists: {}", p.to_string_lossy()),
            Error::InputIsNotDirectory(p) => write!(f, "Input folder is not a directory: {}", p.to_string_lossy()),
            Error::BadTimeDiff(e) => e.fmt(f),
            Error::UnableToCreateDirectory(e) => write!(f, "Unable to create directory: {}", e),
            Error::MessageParseError(e) => write!(f, "Parsing message failed: {}", e),
            Error::UnableToOpenIndex(e) => write!(f, "Unable to open index: {}", e),
            Error::ExitingWithError => write!(f, "Exiting with error. Check output."),
            Error::UnableToOpenFile(e) => write!(f, "Unable to open file: {}", e),
        }
    }
}
