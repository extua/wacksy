use core::error::Error;

#[derive(Debug)]
pub enum IndexingError {
    RecordTimestampError(chrono::ParseError),
    WarcFilenameError(String),
    RecordContentTypeError(String),
    RecordUrlError(url::ParseError),
    RecordStatusError(String),
    ValueNotFound(String),
    UnindexableRecordType(warc::RecordType),
}
impl std::fmt::Display for IndexingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::RecordTimestampError(parse_error_message) => {
                return write!(f, "Could not get record timestamp: {parse_error_message}");
            }
            Self::WarcFilenameError(error_message) => {
                return write!(f, "Could not get WARC filename: {error_message}");
            }
            Self::RecordContentTypeError(error_message) => {
                return write!(f, "Could not parse record content type: {error_message}");
            }
            Self::RecordUrlError(parse_error_message) => {
                return write!(f, "Could not parse url: {parse_error_message}");
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
