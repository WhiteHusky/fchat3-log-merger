use std::{path::PathBuf, str::FromStr, fmt::Display};
use chrono::Duration;

use clap::Parser;
use humantime::{parse_duration, format_duration};

/// Tuple struct containing duration, used for arg parsing.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TimeDiffArg(Duration);

impl TimeDiffArg {
    #[inline(always)]
    fn duration(self) -> Duration {
        self.0
    }
}

impl FromStr for TimeDiffArg {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Duration::from_std(parse_duration(s)?).unwrap()))
    }
}

impl Into<std::time::Duration> for TimeDiffArg {
    fn into(self) -> std::time::Duration {
        self.0.to_std().unwrap()
    }
}

impl From<Duration> for TimeDiffArg {
    #[inline(always)]
    fn from(d: Duration) -> Self {
        Self(d)
    }
}

impl Into<Duration> for TimeDiffArg {
    #[inline(always)]
    fn into(self) -> Duration {
        self.duration()
    }
}

impl Display for TimeDiffArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format_duration(self.0.to_std().unwrap()))
    }
}

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub(crate) struct Args {
    /// What folders to read from
    #[clap(short, long, required = true, min_values = 2)]
    pub(crate) folders: Vec<PathBuf>,

    /// How long the time difference between messages to check for duplicates specified in human time.
    #[clap(short = 'd', long, default_value_t = TimeDiffArg::from(Duration::minutes(5)))]
    pub(crate) time_diff: TimeDiffArg,

    /// Folder to write the merged logs to.
    #[clap(short, long, required_unless_present = "dry-run")]
    pub(crate) output: Option<PathBuf>,

    /// Collects files, but does not do anything.
    #[clap(long)]
    pub(crate) dry_run: bool,

    /// Indicate if a file has more than two duplicate messages in the comparison window.
    #[clap(long)]
    pub(crate) dupe_warning: bool
}