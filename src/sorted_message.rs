use fchat3_log_lib::fchat_message::FChatMessage;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SortedMessage(pub(crate) FChatMessage);

impl From<FChatMessage> for SortedMessage {
    fn from(m: FChatMessage) -> Self {
        Self(m)
    }
}

impl Ord for SortedMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.datetime.cmp(&other.0.datetime)
    }
}

impl PartialOrd for SortedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}