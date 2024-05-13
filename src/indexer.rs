use core::fmt;
use core::str;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

mod cdxj_index_errors;
use cdxj_index_errors::CDXJIndexRecordError;
use chrono::DateTime;
use url::Position;
use url::Url;

use libflate::gzip::MultiDecoder;
use warc::WarcHeader;
use warc::{BufferedBody, Record, RecordIter, RecordType, WarcReader};

pub struct CDXJIndex(Vec<CDXJIndexRecord>);
impl CDXJIndex {
    pub fn new(warc_file_path: &Path) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let mut record_count: usize = 0usize;
        let mut byte_counter: u64 = 0u64;
        let mut index = Vec::with_capacity(1024);

        if warc_file_path.extension() == Some(OsStr::new("gz")) {
            let file_gzip: WarcReader<BufReader<MultiDecoder<BufReader<File>>>> =
                WarcReader::from_path_gzip(warc_file_path)?;
            let file_records: RecordIter<BufReader<MultiDecoder<BufReader<File>>>> =
                file_gzip.iter_records();
            for record in file_records.enumerate() {
                record_count = record.0;
                match record.1 {
                    // Need to be able to skip the record here
                    // add this to a bufwriter
                    Ok(record) => {
                        match CDXJIndexRecord::new(&record, byte_counter, warc_file_path) {
                            Ok(processed_record) => {
                                index.push(processed_record);
                            }
                            Err(err) => eprintln!(
                                "Skipping record number {} with id {} because {err}",
                                record_count,
                                record.warc_id()
                            ),
                        }
                        // here we are getting the length of the record body
                        // in content_length, added to the length of the
                        // unwrapped record header
                        let record_length: u64 = record.content_length()
                            + record.into_raw_parts().0.to_string().len() as u64;
                        // increment the byte counter after processing the record
                        byte_counter = byte_counter.wrapping_add(record_length);
                    }
                    Err(err) => {
                        // Any error with the record at this
                        // point affects the offset counter,
                        // so can't index the rest of the file.
                        eprintln!(
                            "Unable to index the remainder of the file. Record error: {err}\r\n"
                        );
                        break;
                    }
                }
            }
        } else {
            let file_not_gzip: WarcReader<BufReader<File>> = WarcReader::from_path(warc_file_path)?;
            let file_records: RecordIter<BufReader<File>> = file_not_gzip.iter_records();
            for record in file_records.enumerate() {
                record_count = record.0;
                match record.1 {
                    // Need to be able to skip the record here
                    // add this to a bufwriter
                    Ok(record) => {
                        match CDXJIndexRecord::new(&record, byte_counter, warc_file_path) {
                            Ok(processed_record) => {
                                index.push(processed_record);
                            }
                            Err(err) => eprintln!(
                                "Skipping record number {} with id {} because {err}",
                                record_count,
                                record.warc_id()
                            ),
                        }
                        // here we are getting the length of the record body
                        // in content_length, added to the length of the
                        // unwrapped record header
                        let record_length: u64 = record.content_length()
                            + record.into_raw_parts().0.to_string().len() as u64;
                        // increment the byte counter after processing the record
                        byte_counter = byte_counter.wrapping_add(record_length);
                    }
                    Err(err) => {
                        // Any error with the record at this
                        // point affects the offset counter,
                        // so can't index the rest of the file.
                        eprintln!("Unable to index the remainder of the file. Record error: {err}");
                        break;
                    }
                }
            }
        };
        println!("Total records: {record_count}");

        Ok(Self(index))
    }
}

impl Display for CDXJIndex {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let index_string: String = self.0.iter().map(|s| s.to_string()).collect();
        write!(f, "{index_string}")
    }
}

impl CDXJIndexRecord {
    pub fn new(
        record: &Record<BufferedBody>,
        byte_counter: u64,
        warc_file_path: &Path,
    ) -> Result<CDXJIndexRecord, Box<dyn Error + Send + Sync + 'static>> {
        // use something like a control flow enum to
        // organise this
        // https://doc.rust-lang.org/stable/std/ops/enum.ControlFlow.html
        let timestamp = RecordTimestamp::new(record)?;
        let url = RecordUrl::new(record)?;
        let digest = RecordDigest::new(record)?;
        let searchable_url = url.as_searchable_string()?;
        let mime = RecordContentType::new(record)?;
        let status = RecordStatus::new(record)?;
        let filename = WarcFilename::new(record, warc_file_path)?;

        // first check whether the record is either
        // a response, revisit, resource, or metadata
        if [
            RecordType::Response,
            RecordType::Revisit,
            RecordType::Resource,
            RecordType::Metadata,
        ]
        .contains(record.warc_type())
        {
            let parsed_record = CDXJIndexRecord {
                timestamp,
                url,
                searchable_url,
                digest,
                mime,
                filename,
                offset: byte_counter,
                length: record.content_length(),
                status,
            };
            Ok(parsed_record)
        } else {
            Err(format!(
                "Record {} of type {} is not an indexable type, skipping",
                record.warc_id(),
                record.warc_type().to_string()
            )
            .into())
        }
    }
}

#[derive(Debug)]
pub struct CDXJIndexRecord {
    pub timestamp: RecordTimestamp,
    pub searchable_url: String,
    pub url: RecordUrl,          // The URL that was archived
    pub digest: RecordDigest,    // A cryptographic hash for the HTTP response payload
    pub mime: RecordContentType, // The media type for the response payload
    pub filename: WarcFilename,  // The WARC file where the WARC record is located
    pub offset: u64,             // The byte offset for the WARC record
    pub length: u64,             // The length in bytes of the WARC record
    pub status: RecordStatus,    // The HTTP status code for the HTTP response
}
// Display the record as shown in the example in the
// spec https://specs.webrecorder.net/cdxj/0.1.0/#example
// Could there be a better way to serialize this?
impl fmt::Display for CDXJIndexRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "{} {} {{\"url\":\"{}\",\"digest\":\"{}\",\"mime\":\"{}\",\"offset\":{},\"length\":{},\"status\":{},\"filename\":\"{}\"}}",
            self.searchable_url,
            self.timestamp,
            self.url,
            self.digest,
            self.mime,
            self.offset,
            self.length,
            self.status,
            self.filename
        )
    }
}

#[derive(Debug)]
pub struct RecordTimestamp(DateTime<chrono::FixedOffset>);

impl RecordTimestamp {
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, CDXJIndexRecordError> {
        if let Some(warc_header_date) = record.header(WarcHeader::Date) {
            let parsed_datetime: Result<DateTime<chrono::FixedOffset>, chrono::ParseError> =
                DateTime::parse_from_rfc3339(&warc_header_date);
            match parsed_datetime {
                Ok(parsed_datetime) => Ok(Self(parsed_datetime)),
                Err(parsing_error) => {
                    Err(CDXJIndexRecordError::RecordTimestampError(parsing_error))
                }
            }
        } else {
            Err(CDXJIndexRecordError::ValueNotFound(format!(
                "Record {} does not have a date in the WARC header",
                record.warc_id()
            )))
        }
    }
}
impl fmt::Display for RecordTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y%m%d%H%M%S"))
    }
}

#[derive(Debug)]
pub struct WarcFilename(String);

impl WarcFilename {
    pub fn new(
        record: &Record<BufferedBody>,
        warc_file_path: &Path,
    ) -> Result<Self, CDXJIndexRecordError> {
        if let Some(record_filename) = record.header(WarcHeader::Filename) {
            println!("record filename is {record_filename} from file");
            Ok(Self(record_filename.into_owned()))
        } else {
            // If no filename is found in the record
            // we get the filename from the file path
            if let Some(warc_file_path) = warc_file_path.file_name() {
                Ok(Self(warc_file_path.to_string_lossy().to_string()))
            } else {
                // Hit this error case if the filename
                // cannot be inferred from the Path
                Err(CDXJIndexRecordError::WarcFilenameError(format!(
                    "Cannot infer filename from {}",
                    warc_file_path.to_string_lossy()
                )))
            }
        }
    }
}
impl fmt::Display for WarcFilename {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct RecordDigest(String);

impl RecordDigest {
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, CDXJIndexRecordError> {
        if let Some(record_digest) = record.header(WarcHeader::PayloadDigest) {
            Ok(Self(record_digest.to_string()))
        } else {
            Err(CDXJIndexRecordError::ValueNotFound(format!(
                "Record {} does not have a payload digest in the WARC header",
                record.warc_id()
            )))
        }
    }
}
impl fmt::Display for RecordDigest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn cut_http_headers_from_record(record: &Record<BufferedBody>) -> &[u8] {
    // Find the position of the first newline, this will
    // get just the headers, not the full request, see
    // https://stackoverflow.com/questions/69610022/how-can-i-get-httparse-to-parse-my-request-correctly
    let mut first_http_response_byte_counter: usize = 0;
    for byte in record.body() {
        first_http_response_byte_counter += 1;
        if byte == &0xA {
            first_http_response_byte_counter += 1;
            break;
        }
    }

    // Find the position of the first sequence of
    // two newlines, this ends the HTTP 1.1 header block
    // according to section 3 of RFC7230
    let mut second_http_response_byte_counter: usize = 0;
    for byte in record.body() {
        let next_byte: &u8 = record.body().iter().next().unwrap();
        second_http_response_byte_counter += 1;
        if byte == &0xA && next_byte == &0xA {
            break;
        }
    }

    // cut the HTTP header out of the WARC body
    // and, there is an error here to handle
    record
        .body()
        .get(first_http_response_byte_counter..second_http_response_byte_counter)
        .unwrap()
}

#[derive(Debug)]
pub struct RecordContentType(String);

impl RecordContentType {
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, CDXJIndexRecordError> {
        // beware! the warc content type is not the same
        // as the record content type in order to actually
        // do anything about this we need to read
        // the record body
        if record.warc_type() == &RecordType::Revisit {
            // If the WARC record type is revisit,
            // that's the content type
            Ok(Self("revisit".to_owned()))
        } else {
            // create a list of 64 empty headers, if this is not
            // enough then you'll get a TooManyHeaders error
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let header_byte_slice = cut_http_headers_from_record(record);
            // parse the raw byte array with httparse, this adds
            // data to the empty header list created above

            match httparse::parse_headers(header_byte_slice, &mut headers) {
                Ok(headers) => headers,
                Err(http_parsing_error) => {
                    return Err(CDXJIndexRecordError::RecordContentTypeError(
                        http_parsing_error.to_string(),
                    ));
                }
            };

            // loop through the list of headers looking for the content-type
            let mut content_type: Option<Result<&str, std::str::Utf8Error>> = None;
            for header in &headers {
                if header.name == "content-type" {
                    content_type = Some(str::from_utf8(header.value));
                    break;
                }
            }
            if let Some(content_type) = content_type {
                match content_type {
                    Ok(content_type) => Ok(Self(content_type.to_owned())),
                    Err(parsing_error) => Err(CDXJIndexRecordError::RecordContentTypeError(
                        parsing_error.to_string(),
                    )),
                }
            } else {
                Err(CDXJIndexRecordError::ValueNotFound(
                    "could not find content type in HTTP headers".to_owned(),
                ))
            }
        }
    }
}
impl fmt::Display for RecordContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct RecordUrl(Url);

impl RecordUrl {
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, CDXJIndexRecordError> {
        if let Some(warc_header_url) = record.header(WarcHeader::TargetURI) {
            match Url::parse(&warc_header_url) {
                Ok(record_url) => Ok(Self(record_url)),
                Err(parse_error) => Err(CDXJIndexRecordError::RecordUrlError(parse_error)),
            }
        } else {
            Err(CDXJIndexRecordError::ValueNotFound(format!(
                "Record {} does not have a url in the WARC header",
                record.warc_id()
            )))
        }
    }
    pub fn as_searchable_string(&self) -> Result<String, CDXJIndexRecordError> {
        if let Some(host) = self.0.host_str() {
            // split the host string into an array at each dot
            let mut host_split: Vec<&str> = host.split('.').collect();
            // reverse the order of the array
            host_split.reverse();
            // join the array back into a comma-separated string
            let host_reversed = host_split.join(",");
            // capture everything else on the end of the url
            let url_path = &self.0[Position::BeforePath..];
            // put it all together
            Ok(format!("{host_reversed}){url_path}"))
        } else {
            // print the full url here
            let url = self.0.as_str();
            Err(CDXJIndexRecordError::ValueNotFound(format!(
                "Url {url} does not have a host, unable to construct a searchable string"
            )))
        }
    }
}
impl fmt::Display for RecordUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let url_string: String = self.0.clone().into();
        write!(f, "{}", url_string.to_lowercase())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordStatus(u16);

impl RecordStatus {
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, CDXJIndexRecordError> {
        // Cut a slice out from the record body from
        // byte 9 to byte 12, this should be the
        // status code
        let header_byte_slice: &[u8] = &record.body()[9..12];
        // Convert it to a string, if this doesn't work
        // it'll produce unknown characters, this could
        // be a properly-handled error?
        let header_status = String::from_utf8_lossy(header_byte_slice);
        // Parse it to a number, if there's an error it appears here!
        match header_status.parse::<u16>() {
            Ok(header_status_int) => Ok(Self(header_status_int)),
            Err(int_error) => Err(CDXJIndexRecordError::RecordStatusError(int_error)),
        }
    }
}
impl fmt::Display for RecordStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
