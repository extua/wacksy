//! Reads the WARC file and composes a CDX(J) index.

use core::{fmt, str};
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

mod cdxj_index_errors;
use cdxj_index_errors::CDXJIndexRecordError;
use chrono::DateTime;
use url::{Position, Url};

use libflate::gzip::MultiDecoder;
use warc::WarcHeader;
use warc::{BufferedBody, Record, RecordIter, RecordType, WarcReader};

pub struct CDXJIndex(Vec<CDXJIndexRecord>);
impl CDXJIndex {
    /// # Create new index
    ///
    /// This is the main function which sets off looping
    /// over the records and building the index.
    ///
    /// # Errors
    ///
    /// Will return a `std::io::Error` from
    /// `WarcReader::from_path`/`from_path_gzip`
    /// in case of any problem reading the Warc file.
    pub fn new(warc_file_path: &Path) -> Result<Self, std::io::Error> {
        // this looping function accepts a generic type which
        // this allows us to pass in both gzipped and non-gzipped records
        fn loop_over_records<T: Iterator<Item = Result<Record<BufferedBody>, warc::Error>>>(
            file_records: T,
            warc_file_path: &Path,
        ) -> CDXJIndex {
            let mut record_count: usize = 0;
            let mut byte_counter: u64 = 0;
            let mut index = Vec::with_capacity(1024);

            for record in file_records.enumerate() {
                record_count = record.0;
                match record.1 {
                    // add this to a bufwriter
                    Ok(record) => {
                        match CDXJIndexRecord::new(&record, byte_counter, warc_file_path) {
                            Ok(processed_record) => {
                                index.push(processed_record);
                            }
                            Err(err) => eprintln!(
                                // Any error with the record means we have to
                                // skip over it and move on to the next one.
                                "Skipping record number {record_count} with id {}: {err}",
                                record.warc_id()
                            ),
                        }
                        // Get the length of the record body in content_length,
                        // added to the length of the unwrapped record header
                        let record_length: u64 = record.content_length()
                            + record.into_raw_parts().0.to_string().len() as u64;

                        // increment the byte counter after processing the record
                        byte_counter = byte_counter.wrapping_add(record_length);
                    }
                    Err(err) => {
                        // Any error with the record here affects the offset counter,
                        // so can't index the rest of the file.
                        eprintln!(
                            "Unable to index the remainder of the file. WARC header parsing error: {err}"
                        );
                        break;
                    }
                }
            }
            println!("Total records: {record_count}");

            return CDXJIndex(index);
        }

        if warc_file_path.extension() == Some(OsStr::new("gz")) {
            let file_gzip: WarcReader<BufReader<MultiDecoder<BufReader<File>>>> =
                WarcReader::from_path_gzip(warc_file_path)?;
            let file_records: RecordIter<BufReader<MultiDecoder<BufReader<File>>>> =
                file_gzip.iter_records();
            return Ok(loop_over_records(file_records, warc_file_path));
        } else {
            let file_not_gzip: WarcReader<BufReader<File>> = WarcReader::from_path(warc_file_path)?;
            let file_records: RecordIter<BufReader<File>> = file_not_gzip.iter_records();
            return Ok(loop_over_records(file_records, warc_file_path));
        };
    }
}

impl fmt::Display for CDXJIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let index_string: String = self.0.iter().map(ToString::to_string).collect();
        return write!(f, "{index_string}");
    }
}

/// A page which would make up
/// a line in a pages.jsonl file.
#[derive(Debug)]
pub struct PageRecord {
    /// The date and time when the web archive snapshot was created
    pub timestamp: RecordTimestamp,
    /// The URL that was archived
    pub url: RecordUrl,
    /// A string describing the resource
    pub title: Option<PageTitle>,
}

/// A record which would make up
/// a line in a CDX(J) index.
#[derive(Debug)]
pub struct CDXJIndexRecord {
    /// The date and time when the web archive snapshot was created
    pub timestamp: RecordTimestamp,
    /// Sort-friendly formatted URL
    pub searchable_url: String,
    /// The URL that was archived
    pub url: RecordUrl,
    /// A cryptographic hash for the HTTP response payload       
    pub digest: RecordDigest,
    /// The media type for the response payload
    pub mime: RecordContentType,
    /// The WARC file where the WARC record is located
    pub filename: WarcFilename,
    /// The byte offset for the WARC record
    pub offset: u64,
    /// The length in bytes of the WARC record
    pub length: u64,
    // The HTTP status code for the HTTP response
    pub status: RecordStatus,
}

impl CDXJIndexRecord {
    /// # Create index record
    ///
    /// Takes a `Record<BufferedBody>` and parses it to extract all
    /// the fields which make up a CDX(J) record.
    ///
    /// # Errors
    ///
    /// If the record is not a Warc `response`, `revisit`, `resource`, or `metadata`,
    /// an `UnindexableRecordType` error is returned. Otherwise, returns corresponding
    /// errors for each of the CDX(J) fields.
    pub fn new(
        record: &Record<BufferedBody>,
        byte_counter: u64,
        warc_file_path: &Path,
    ) -> Result<Self, CDXJIndexRecordError> {
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
            let parsed_record = Self {
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
            return Ok(parsed_record);
        } else {
            // if the record is not one of the types we want,
            // return an error
            let warc_type = record.warc_type().clone();
            return Err(CDXJIndexRecordError::UnindexableRecordType(warc_type));
        }
    }
}

// Display the record as shown in the example in the
// spec https://specs.webrecorder.net/cdxj/0.1.0/#example
// Could there be a better way to serialize this?
impl fmt::Display for CDXJIndexRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return writeln!(
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
        );
    }
}

#[derive(Debug)]
pub struct RecordTimestamp(DateTime<chrono::FixedOffset>);

impl RecordTimestamp {
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, CDXJIndexRecordError> {
        match record.header(WarcHeader::Date) {
            Some(warc_header_date) => match DateTime::parse_from_rfc3339(&warc_header_date) {
                Ok(parsed_datetime) => return Ok(Self(parsed_datetime)),
                Err(parsing_error) => {
                    return Err(CDXJIndexRecordError::RecordTimestampError(parsing_error));
                }
            },
            None => {
                return Err(CDXJIndexRecordError::ValueNotFound(format!(
                    "Record {} does not have a date in the WARC header",
                    record.warc_id()
                )));
            }
        }
    }
}
impl fmt::Display for RecordTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}", self.0.format("%Y%m%d%H%M%S"));
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
            return Ok(Self(record_filename.into_owned()));
        } else {
            // If no filename is found in the record
            // we get the filename from the file path
            if let Some(warc_file_path) = warc_file_path.file_name() {
                return Ok(Self(warc_file_path.to_string_lossy().to_string()));
            } else {
                // Hit this error case if the filename
                // cannot be inferred from the Path
                return Err(CDXJIndexRecordError::WarcFilenameError(format!(
                    "Cannot infer filename from {}",
                    warc_file_path.to_string_lossy()
                )));
            }
        }
    }
}
impl fmt::Display for WarcFilename {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}", self.0);
    }
}

#[derive(Debug)]
pub struct RecordDigest(String);

impl RecordDigest {
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, CDXJIndexRecordError> {
        if let Some(record_digest) = record.header(WarcHeader::PayloadDigest) {
            return Ok(Self(record_digest.to_string()));
        } else {
            return Err(CDXJIndexRecordError::ValueNotFound(format!(
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
            return Ok(Self("revisit".to_owned()));
        } else {
            // create a list of 64 empty headers, if this is not
            // enough then you'll get a TooManyHeaders error
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut response = httparse::Response::new(&mut headers);
            match response.parse(record.body()) {
                Ok(status) => status,
                Err(http_parsing_error) => {
                    return Err(CDXJIndexRecordError::RecordContentTypeError(
                        http_parsing_error.to_string(),
                    ));
                }
            };

            // loop through the list of headers looking for the content-type
            let mut content_type: Option<Result<&str, str::Utf8Error>> = None;
            for header in &headers {
                if header.name == "content-type" {
                    content_type = Some(str::from_utf8(header.value));
                    break;
                }
            }
            match content_type {
                Some(some_content_type) => match some_content_type {
                    Ok(parsed_content_type) => return Ok(Self(parsed_content_type.to_owned())),
                    Err(parsing_error) => {
                        return Err(CDXJIndexRecordError::RecordContentTypeError(
                            parsing_error.to_string(),
                        ));
                    }
                },
                None => {
                    return Err(CDXJIndexRecordError::ValueNotFound(
                        "could not find content type in HTTP headers".to_owned(),
                    ));
                }
            };
        }
    }
}
impl fmt::Display for RecordContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}", self.0);
    }
}

#[derive(Debug)]
pub struct RecordUrl(Url);

impl RecordUrl {
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, CDXJIndexRecordError> {
        if let Some(warc_header_url) = record.header(WarcHeader::TargetURI) {
            match Url::parse(&warc_header_url) {
                Ok(record_url) => return Ok(Self(record_url)),
                Err(parse_error) => return Err(CDXJIndexRecordError::RecordUrlError(parse_error)),
            }
        } else {
            return Err(CDXJIndexRecordError::ValueNotFound(
                "TargetURI not present in the WARC header".to_owned(),
            ));
        }
    }
    /// Compose searchable string
    ///
    /// Take a url and return a Sort-friendly URI Reordering Transform (SURT)
    /// formatted string. It is cast to lowercase when displayed.
    ///
    /// Errors
    ///
    /// Will return `ValueNotFound` if the url does not have a host.
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
            return Ok(format!("{host_reversed}){url_path}"));
        } else {
            // print the full url here
            let url = self.0.as_str();
            return Err(CDXJIndexRecordError::ValueNotFound(format!(
                "{url} does not have a host, unable to construct a searchable string"
            )));
        }
    }
}
impl fmt::Display for RecordUrl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let url_string: String = self.0.clone().into();
        return write!(f, "{}", url_string.to_lowercase());
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordStatus(u16);

impl RecordStatus {
    /// # Record status
    ///
    /// Parse the record body with httparse and get
    /// the status code from the response.
    ///
    /// # Errors
    ///
    /// Will return a `RecordStatusError`,
    /// which can contain either a _parsing_ error from httparse,
    /// or an error arising from an empty response code.
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, CDXJIndexRecordError> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut response = httparse::Response::new(&mut headers);

        match response.parse(record.body()) {
            Ok(_) => match response.code {
                Some(response_code) => return Ok(Self(response_code)),
                None => {
                    return Err(CDXJIndexRecordError::RecordStatusError(
                        "response code is empty".to_owned(),
                    ));
                }
            },
            Err(http_parsing_error) => {
                return Err(CDXJIndexRecordError::RecordStatusError(
                    http_parsing_error.to_string(),
                ));
            }
        };
    }
}
impl fmt::Display for RecordStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}", self.0);
    }
}

#[derive(Debug)]
pub struct PageTitle(String);

// impl PageTitle {
//     pub fn new(record: &Record<BufferedBody>) -> Result<Self, CDXJIndexRecordError> {
//         if let Some(record_digest) = record.header(WarcHeader::PayloadDigest) {
//             return Ok(Self(record_digest.to_string()));
//         } else {
//             return Err(CDXJIndexRecordError::ValueNotFound(format!(
//                 "Record {} does not have a payload digest in the WARC header",
//                 record.warc_id()
//             )));
//         }
//     }
// }
impl fmt::Display for PageTitle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}", self.0);
    }
}