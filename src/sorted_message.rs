use fchat3_log_lib::fchat_message::FChatMessage;

pub(crate) struct SortedMessage {
    pub(crate) message: FChatMessage
}

impl Ord for SortedMessage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.message.datetime.cmp(&other.message.datetime)
    }
}

impl PartialOrd for SortedMessage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SortedMessage {
    fn eq(&self, other: &Self) -> bool {
        self.message.datetime == other.message.datetime
    }
}

impl Eq for SortedMessage {
    fn assert_receiver_is_total_eq(&self) { unimplemented!() }
}
