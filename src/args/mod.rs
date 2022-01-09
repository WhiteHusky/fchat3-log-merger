use std::path::PathBuf;
use chrono::Duration;
use clap::Parser;

mod time_diff_arg;
use time_diff_arg::TimeDiffArg;

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