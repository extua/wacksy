use std::error::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum IndexingError {
    /// could not read timestamp from record
    RecordTimestampError(chrono::ParseError),
    /// could not get WARC filename
    WarcFilenameError(String),
    /// could not parse record content type
    RecordContentTypeError(String),
    /// could nor parse the target url
    RecordUrlError(url::ParseError),
    /// could not parse HTTP status code
    RecordStatusError(String),
    /// some value was missing
    ValueNotFound(String),
    /// this type of record can not be indexed
    UnindexableRecordType(warc::RecordType),
}
impl Display for IndexingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::RecordTimestampError(parse_error_message) => {
                return write!(f, "Could not read timestamp from record: {parse_error_message}");
            }
            Self::WarcFilenameError(error_message) => {
                return write!(f, "Could not get WARC filename: {error_message}");
            }
            Self::RecordContentTypeError(error_message) => {
                return write!(f, "Could not parse record content type: {error_message}");
            }
            Self::RecordUrlError(parse_error_message) => {
                return write!(f, "Could not parse target url: {parse_error_message}");
            }
            Self::RecordStatusError(parse_int_error_message) => {
                return write!(f, "Could not parse HTTP status: {parse_int_error_message}");
            }
            Self::ValueNotFound(error_message) => {
                return write!(f, "Value not found: {error_message}");
            }
            Self::UnindexableRecordType(warc_type) => {
                return write!(
                    f,
                    "Could not index this type of record: {}",
                    warc_type.to_string()
                );
            }
        }
    }
}
impl Error for IndexingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RecordTimestampError(parse_error) => return Some(parse_error),
            Self::RecordUrlError(parse_error) => return Some(parse_error),
            Self::ValueNotFound(_)
            | Self::RecordStatusError(_)
            | Self::UnindexableRecordType(_)
            | Self::RecordContentTypeError(_)
            | Self::WarcFilenameError(_) => return None,
        }
    }
}
