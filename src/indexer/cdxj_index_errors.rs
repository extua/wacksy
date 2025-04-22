use std::error::Error;

#[derive(Debug)]
pub enum CDXJIndexRecordError {
    RecordTimestampError(chrono::ParseError),
    WarcFilenameError(String),
    RecordContentTypeError(String),
    RecordUrlError(url::ParseError),
    RecordStatusError(std::num::ParseIntError),
    ValueNotFound(String),
}
impl std::fmt::Display for CDXJIndexRecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CDXJIndexRecordError::RecordTimestampError(parse_error_message) => {
                write!(f, "Could not get record timestamp: {parse_error_message}")
            }
            CDXJIndexRecordError::WarcFilenameError(error_message) => {
                write!(f, "Could not get WARC filename: {error_message}")
            }
            CDXJIndexRecordError::RecordContentTypeError(error_message) => {
                write!(f, "Could not parse record content type: {error_message}")
            }
            CDXJIndexRecordError::RecordUrlError(parse_error_message) => {
                write!(f, "Could not parse url: {parse_error_message}")
            }
            CDXJIndexRecordError::RecordStatusError(parse_int_error_message) => {
                write!(f, "Could not parse HTTP status: {parse_int_error_message}")
            }
            CDXJIndexRecordError::ValueNotFound(error_message) => {
                write!(f, "Value not found: {error_message}")
            }
        }
    }
}
impl Error for CDXJIndexRecordError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CDXJIndexRecordError::RecordTimestampError(parse_error) => Some(parse_error),
            CDXJIndexRecordError::RecordUrlError(parse_error) => Some(parse_error),
            CDXJIndexRecordError::RecordStatusError(parse_int_error) => Some(parse_int_error),
            CDXJIndexRecordError::ValueNotFound(_)
            | CDXJIndexRecordError::RecordContentTypeError(_)
            | CDXJIndexRecordError::WarcFilenameError(_) => None,
        }
    }
}
