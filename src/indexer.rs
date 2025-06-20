//! Reads the WARC file and composes a CDX(J) index.

use core::{fmt, str};
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

mod indexing_errors;
use chrono::DateTime;
use indexing_errors::IndexingError;
use serde::{Deserialize, Serialize};
use url::{Position, Url};

use libflate::gzip::MultiDecoder;
use warc::WarcHeader;
use warc::{BufferedBody, Record, RecordIter, RecordType, WarcReader};

/// # Indexer
///
/// This function sets off looping through the
/// records to build the index and create the
/// pages.jsonl file.
///
/// # Errors
///
/// Will return a `std::io::Error` from
/// `WarcReader::from_path`/`from_path_gzip`
/// in case of any problem reading the Warc file.
pub fn index_file(warc_file_path: &Path) -> Result<Index, std::io::Error> {
    // this looping function accepts a generic type which
    // this allows us to pass in both gzipped and non-gzipped records
    fn loop_over_records<T: Iterator<Item = Result<Record<BufferedBody>, warc::Error>>>(
        file_records: T,
        warc_file_path: &Path,
    ) -> Index {
        let mut record_count: usize = 0;
        let mut byte_counter: u64 = 0;
        let mut cdxj_index: Vec<CDXJIndexRecord> = Vec::with_capacity(1024);
        let mut page_index: Vec<PageRecord> = Vec::with_capacity(1024);

        for record in file_records.enumerate() {
            record_count = record.0;
            match record.1 {
                Ok(record) => {
                    match CDXJIndexRecord::new(&record, byte_counter, warc_file_path) {
                        Ok(processed_record) => {
                            cdxj_index.push(processed_record);
                        }
                        Err(err) => eprintln!(
                            // Any error with the record means we have to
                            // skip over it and move on to the next one.
                            "Skipping record number {record_count} with id {}: {err}",
                            record.warc_id()
                        ),
                    }
                    match PageRecord::new(&record) {
                        Ok(processed_record) => {
                            page_index.push(processed_record);
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

        return Index(CDXJIndex(cdxj_index), PageIndex(page_index));
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

pub struct Index(pub CDXJIndex, pub PageIndex);

pub struct CDXJIndex(Vec<CDXJIndexRecord>);
impl fmt::Display for CDXJIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let index_string: String = self.0.iter().map(ToString::to_string).collect();
        return write!(f, "{index_string}");
    }
}

pub struct PageIndex(Vec<PageRecord>);
impl fmt::Display for PageIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let index_string: String = self.0.iter().map(ToString::to_string).collect();
        return write!(f, "{index_string}");
    }
}

/// A page which would make up a line in a pages.jsonl file.
#[derive(Debug, Serialize)]
pub struct PageRecord {
    /// The date and time when the web archive snapshot was created
    pub timestamp: RecordTimestamp,
    /// The URL that was archived
    pub url: RecordUrl,
    /// A string describing the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<PageTitle>,
}
impl PageRecord {
    /// # Create page record
    ///
    /// Takes a `Record<BufferedBody>` and extracts the
    /// timestamp and url for the pages.jsonl file.
    ///
    /// # Errors
    ///
    /// Returns an `UnindexableRecordType` error if the record is not
    /// a Warc `response`, `revisit`, or `resource`. Otherwise, returns
    /// corresponding errors for url and timestamp fields.
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, IndexingError> {
        let timestamp = RecordTimestamp::new(record)?;
        let url = RecordUrl::new(record)?;

        // first check whether the record is either
        // a response, revisit, resource, or metadata
        if [
            RecordType::Response,
            RecordType::Revisit,
            RecordType::Resource,
        ]
        .contains(record.warc_type())
        {
            let parsed_record = Self {
                timestamp,
                url,
                title: None,
            };
            return Ok(parsed_record);
        } else {
            // if the record is not one of the types we want,
            // return an error
            let warc_type = record.warc_type().clone();
            // change this to a generic indexing error?
            return Err(IndexingError::UnindexableRecordType(warc_type));
        }
    }
}
/// Display the record to json.
impl fmt::Display for PageRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pages_json_string = serde_json::to_string(self).unwrap();
        return writeln!(f, "{pages_json_string}");
    }
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
    /// # Create CDXJ index record
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
    ) -> Result<Self, IndexingError> {
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
            return Err(IndexingError::UnindexableRecordType(warc_type));
        }
    }
}

/// Display the record to json as shown in [the example in the
/// spec](https://specs.webrecorder.net/cdxj/0.1.0/#example)
///
/// Could there be a better way to serialize this?
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

#[derive(Debug, Deserialize, Serialize)]
pub struct RecordTimestamp(DateTime<chrono::FixedOffset>);

impl RecordTimestamp {
    /// # Get timestamp
    ///
    /// Get the timestamp from the WARC header field `WarcHeader::Date`,
    /// parse it to a `DateTime<FixedOffset>`.
    ///
    /// # Errors
    ///
    /// Returns a `RecordTimestampError` if there is a problem with
    /// parsing, and this wraps `chrono::ParseError`. Otherwise returns
    /// `ValueNotFound` if there is no date in the WARC header.
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, IndexingError> {
        match record.header(WarcHeader::Date) {
            Some(warc_header_date) => match DateTime::parse_from_rfc3339(&warc_header_date) {
                Ok(parsed_datetime) => return Ok(Self(parsed_datetime)),
                Err(parsing_error) => {
                    return Err(IndexingError::RecordTimestampError(parsing_error));
                }
            },
            None => {
                return Err(IndexingError::ValueNotFound(format!(
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
    /// # Create Warc filename
    ///
    /// Takes the filename from `WarcHeader::Filename`, and converts it
    /// to a string. If no filename is found in the record this function
    /// reads the path to the warc file.
    ///
    /// # Errors
    ///
    /// Returns a `WarcFilenameError` error if the filename cannot be
    /// inferred from the file path. Normally you should not hit this
    /// error.
    pub fn new(
        record: &Record<BufferedBody>,
        warc_file_path: &Path,
    ) -> Result<Self, IndexingError> {
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
                return Err(IndexingError::WarcFilenameError(format!(
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

#[derive(Debug)]
pub struct RecordContentType(String);

impl RecordContentType {
    /// # Parse record content type
    ///
    /// Parses the HTTP content type from the HTTP headers in
    /// the record body; this is not the same as the
    /// [content type from the WARC header](https://iipc.github.io/warc-specifications/specifications/warc-format/warc-1.1/#content-type),
    /// which would ususally be `application/http`.
    ///
    /// If the WARC record type is `revisit`, in which case the spec
    /// says to directly return that as the content type.
    ///
    /// # Errors
    ///
    /// Returns a `RecordContentTypeError` in case of any problems with
    /// parsing; this either wraps `httparse::Error`, or a `Utf8Error` when
    /// parsing the content type to string. Alternatively returns `ValueNotFound`
    /// if no content type is found in the HTTP headers.
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, IndexingError> {
        if record.warc_type() == &RecordType::Revisit {
            return Ok(Self("revisit".to_owned()));
        } else {
            // create a list of 64 empty headers, if this is not
            // enough then you'll get a TooManyHeaders error
            let mut headers = [httparse::EMPTY_HEADER; 64];
            let mut response = httparse::Response::new(&mut headers);
            match response.parse(record.body()) {
                Ok(status) => status,
                Err(http_parsing_error) => {
                    return Err(IndexingError::RecordContentTypeError(
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
                        return Err(IndexingError::RecordContentTypeError(
                            parsing_error.to_string(),
                        ));
                    }
                },
                None => {
                    return Err(IndexingError::ValueNotFound(
                        "content type not present in HTTP headers".to_owned(),
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

#[derive(Debug, Serialize)]
pub struct RecordUrl(Url);

impl RecordUrl {
    /// # Get the url of the record
    ///
    /// Get the url from the `WarcHeader::TargetURI` field.
    ///
    /// # Errors
    ///
    /// Returns `RecordUrlError` if there is any problem parsing
    /// the Url, this is a wrapper for `url::ParseError`.
    /// Alternatively returns `ValueNotFound` if no `TargetURI` field
    /// is present in the WARC header.
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, IndexingError> {
        if let Some(warc_header_url) = record.header(WarcHeader::TargetURI) {
            match Url::parse(&warc_header_url) {
                Ok(record_url) => return Ok(Self(record_url)),
                Err(parse_error) => return Err(IndexingError::RecordUrlError(parse_error)),
            }
        } else {
            return Err(IndexingError::ValueNotFound(
                "TargetURI not present in the WARC header".to_owned(),
            ));
        }
    }
    /// # Compose searchable string
    ///
    /// Take a url and return a Sort-friendly URI Reordering Transform (SURT)
    /// formatted string. It is cast to lowercase when displayed.
    ///
    /// # Errors
    ///
    /// Returns `ValueNotFound` if the url does not have a host.
    pub fn as_searchable_string(&self) -> Result<String, IndexingError> {
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
            return Err(IndexingError::ValueNotFound(format!(
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
    /// # Parse record status
    ///
    /// Parse the record body with httparse and get the status code
    /// from the response.
    ///
    /// # Errors
    ///
    /// Returns a `RecordStatusError`, which can contain either
    /// a _parsing_ error from httparse, or an error arising
    /// from an empty response code.
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, IndexingError> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut response = httparse::Response::new(&mut headers);

        match response.parse(record.body()) {
            Ok(_) => match response.code {
                Some(response_code) => return Ok(Self(response_code)),
                None => {
                    return Err(IndexingError::RecordStatusError(
                        "response code is empty".to_owned(),
                    ));
                }
            },
            Err(http_parsing_error) => {
                return Err(IndexingError::RecordStatusError(
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

// This has not been properly implemented yet!
#[doc(hidden)]
#[derive(Debug, Serialize, Deserialize)]
pub struct PageTitle(String);

impl PageTitle {
    // pub fn new(record: &Record<BufferedBody>) -> Result<Self, IndexingError> {
    //     if let Some(record_digest) = record.header(WarcHeader::PayloadDigest) {
    //         return Ok(Self(record_digest.to_string()));
    //     } else {
    //         return Err(IndexingError::ValueNotFound(format!(
    //             "Record {} does not have a payload digest in the WARC header",
    //             record.warc_id()
    //         )));
    //     }
    // }
}
impl fmt::Display for PageTitle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(f, "{}", self.0);
    }
}
