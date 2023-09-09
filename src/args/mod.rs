use std::path::PathBuf;
use chrono::Duration;
use clap::Parser;

mod time_diff_arg;
use time_diff_arg::TimeDiffArg;
mod timestamp_arg;
use timestamp_arg::TimestampArg;

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(
    help_template = "{name} {version} \n{author-with-newline} {about-section} \n {usage-heading} {usage} \n {all-args} {tab}"
)]
pub(crate) struct Args {
    /// What folders to read from.
    #[clap(short, long, required = true, num_args = 2..)]
    pub(crate) folders: Vec<PathBuf>,

    /// How long the time difference between messages to check for duplicates specified in human time.
    #[clap(short = 'd', long, default_value_t = TimeDiffArg::from(Duration::zero()))]
    pub(crate) time_diff: TimeDiffArg,

    /// Assuming the left-most is up-to-date, skip to this timestamp in YYYY-MM-DD HH:MM:SS.
    #[clap(long)]
    pub(crate) fast_forward: Option<TimestampArg>,

    /// Folder to write the merged logs to.
    #[clap(short, long, required_unless_present = "dry_run")]
    pub(crate) output: Option<PathBuf>,

    /// Collects files, but does not do anything.
    #[clap(long)]
    pub(crate) dry_run: bool,

    /// Indicate if a file has more than one duplicate messages in the comparison window.
    #[clap(long)]
    pub(crate) dupe_warning: bool,

    /// Increase verbosity. More occurances increases the verbosity.
    #[clap(short, action = clap::ArgAction::Count)]
    pub(crate) verbosity: u8
}