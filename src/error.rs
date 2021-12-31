use std::io;

use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum Error {
    OutputExists(PathBuf),
    NotEnoughInputs,
    InputDoesNotExist(PathBuf),
    InputIsNotDirectory(PathBuf),
    BadTimeDiff(humantime::DurationError),
    UnableToCreateDirectory(PathBuf, io::Error),
    MessageParseError(fchat3_log_lib::error::Error),
    UnableToOpenIndex(PathBuf, io::Error),
    UnableToOpenFile(PathBuf, io::Error),
    UnableToOpenDirectory(PathBuf, io::Error),
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
            Error::OutputExists(p) =>
                write!(f, "Output folder already exists: {}", p.to_string_lossy()),
            Error::NotEnoughInputs =>
                write!(f, "Specify more than one input folder."),
            Error::InputDoesNotExist(p) =>
                write!(f, "Input folder does not exists: {}", p.to_string_lossy()),
            Error::InputIsNotDirectory(p) =>
                write!(f, "Input folder is not a directory: {}", p.to_string_lossy()),
            Error::BadTimeDiff(e) =>
                e.fmt(f),
            Error::UnableToCreateDirectory(p, e) =>
                write!(f, "Unable to create directory `{}`: {}", p.display(), e),
            Error::MessageParseError(e) =>
                write!(f, "Parsing message failed: {}", e),
            Error::UnableToOpenIndex(p, e) =>
                write!(f, "Unable to open index `{}` due to: {}", p.display(), e),
            Error::ExitingWithError =>
                write!(f, "Exiting with error. Check output."),
            Error::UnableToOpenFile(p, e) =>
                write!(f, "Unable to open file `{}` due to: {}", p.display(), e),
            Error::UnableToOpenDirectory(p, e) =>
                write!(f, "Unable to open directory `{}` due to: {}", p.display(), e),
        }
    }
}
