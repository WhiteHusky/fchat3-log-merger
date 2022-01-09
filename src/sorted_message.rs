use fchat3_log_lib::fchat_message::FChatMessage;

#[derive(Debug)]
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

impl PartialEq for SortedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.0.datetime == other.0.datetime
    }
}

impl Eq for SortedMessage {
    fn assert_receiver_is_total_eq(&self) { unimplemented!() }
}
