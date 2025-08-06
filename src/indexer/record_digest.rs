use crate::indexer::indexing_errors::IndexingError;
use std::fmt;
use warc::{BufferedBody, Record, WarcHeader};

#[derive(Debug)]
pub struct RecordDigest(String);

impl RecordDigest {
    /// # Get Warc digest
    ///
    /// Takes the digest from from `WarcHeader::PayloadDigest`, and
    /// returns it as a string.
    ///
    /// # Errors
    ///
    /// Returns a `ValueNotFound` error if no payload digest is found
    /// in the WARC header.
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, IndexingError> {
        if let Some(record_digest) = record.header(WarcHeader::PayloadDigest) {
            return Ok(Self(record_digest.to_string()));
        } else {
            return Err(IndexingError::ValueNotFound(format!(
                "Record {} does not have a payload digest in the WARC header",
                record.warc_id()
            )));
        }
    }
}
impl fmt::Display for RecordDigest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}", self.0);
    }
}
