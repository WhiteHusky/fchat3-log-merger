use std::fmt::Display;
use std;
use humantime::{parse_duration, format_duration};
use std::str::FromStr;
use chrono::Duration;

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
