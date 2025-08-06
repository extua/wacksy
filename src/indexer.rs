//! Reads the WARC file and composes a CDX(J) index.

use libflate::gzip::MultiDecoder;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use warc::{BufferedBody, Record, RecordIter, RecordType, WarcReader};

mod indexing_errors;
pub use indexing_errors::IndexingError;
mod page_record;
pub use page_record::PageRecord;
mod record_timestamp;
pub use record_timestamp::RecordTimestamp;
mod warc_filename;
pub use warc_filename::WarcFilename;
mod record_digest;
pub use record_digest::RecordDigest;
mod record_content_type;
pub use record_content_type::RecordContentType;
mod record_url;
pub use record_url::RecordUrl;
mod record_status;
pub use record_status::RecordStatus;

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
                        + u64::try_from(record.into_raw_parts().0.to_string().len()).unwrap();

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

/// This index struct contains a list of individual [CDX(J) Records](CDXJIndexRecord).
pub struct CDXJIndex(Vec<CDXJIndexRecord>);
impl fmt::Display for CDXJIndex {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        let index_string: String = self.0.iter().map(ToString::to_string).collect();
        return write!(message, "{index_string}");
    }
}

pub struct PageIndex(Vec<PageRecord>);
impl fmt::Display for PageIndex {
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        let index_string: String = self.0.iter().map(ToString::to_string).collect();
        return write!(message, "{index_string}");
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
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        return writeln!(
            message,
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
    fn fmt(&self, message: &mut fmt::Formatter) -> fmt::Result {
        return write!(message, "{}", self.0);
    }
}
