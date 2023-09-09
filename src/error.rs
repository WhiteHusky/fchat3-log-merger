use std::io;

use std::path::PathBuf;
use thiserror::Error as E;

#[derive(E, Debug)]
pub(crate) enum Error {
    #[error("Output folder `{0}` already exists")]
    OutputExists(PathBuf),
    #[error("Specify more than one input folder")]
    NotEnoughInputs,
    #[error("Input folder `{0}` does not exist")]
    InputDoesNotExist(PathBuf),
    #[error("Input folder `{0}` is not a directory")]
    InputIsNotDirectory(PathBuf),
    #[error("{0}")]
    BadTimeDiff(#[from] humantime::DurationError),
    #[error("{0}")]
    BadTimestamp(#[from] humantime::TimestampError),
    #[error("Unable to create directory `{0}` due to: {1}")]
    UnableToCreateDirectory(PathBuf, io::Error),
    #[error("Failed to parse a message due to: {0}")]
    MessageParseError(#[from] fchat3_log_lib::error::Error),
    #[error("Unable to open index `{0}` due to: {1}")]
    UnableToOpenIndex(PathBuf, io::Error),
    #[error("Unable to open log `{0}` due to: {1}")]
    UnableToOpenLog(PathBuf, io::Error),
    #[error("Unable to open directory `{0}` due to: {1}")]
    UnableToOpenDirectory(PathBuf, io::Error),
    #[error("Exiting with error. Check output.")]
    ExitingWithError
}
