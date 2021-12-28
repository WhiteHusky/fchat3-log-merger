use std::io::BufReader;

use fchat3_log_lib::read_fchatmessage_from_buf;
use fchat3_log_lib::fchat_message::FChatMessage;
use fchat3_log_lib::ReadSeek;

use crate::Error;


pub(crate) struct Reader<'a> {
    pub(crate) buf: Box<dyn ReadSeek + 'a>
}

impl<'a> Reader<'a> {
    pub(crate) fn new<T: 'a + ReadSeek>(stream: T) -> Self {
        Self { buf: Box::new(stream) }
    }

    pub(crate) fn new_buffered<T: 'a + ReadSeek>(stream: T) -> Self {
        let br = BufReader::new(stream);
        Self::new(br)
    }
}

impl Iterator for Reader<'_> {
    type Item = Result<FChatMessage, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match read_fchatmessage_from_buf(&mut self.buf) {
            Ok(Some(m)) => Some(Ok(m)),
            Ok(None) => None,
            Err(e) => Some(Err(Error::from(e))),
        }
    }
}
