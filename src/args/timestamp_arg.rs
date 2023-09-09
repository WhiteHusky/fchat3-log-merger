use std::fmt::Display;
use std;
use std::time::UNIX_EPOCH;
use humantime::Timestamp;
use std::str::FromStr;
use chrono::NaiveDateTime;

/// Tuple struct containing timestamp, used for arg parsing.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TimestampArg(NaiveDateTime);

impl TimestampArg {
    #[inline(always)]
    fn timestamp(self) -> NaiveDateTime {
        self.0
    }
}

impl FromStr for TimestampArg {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let unix_time = Timestamp::from_str(s)?.duration_since(UNIX_EPOCH).unwrap().as_secs();
        Ok(Self(NaiveDateTime::from_timestamp_opt(unix_time as i64, 0).unwrap()))
    }
}

impl From<NaiveDateTime> for TimestampArg {
    fn from(d: NaiveDateTime) -> Self {
        Self(d)
    }
}

impl Into<NaiveDateTime> for TimestampArg {
    fn into(self) -> NaiveDateTime {
        self.timestamp()
    }
}

impl Display for TimestampArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = self.0;
        write!(f, "{}", d.format("%Y-%m-%d %H:%M:%S"))
    }
}
