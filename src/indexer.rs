use core::error::Error;
use core::str;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use chrono::DateTime;
use libflate::gzip::MultiDecoder;
use url::{Position, Url};
use warc::{BufferedBody, Record, RecordIter, RecordType, WarcHeader, WarcReader};

pub struct CDXJIndexRecord {
    url: Url,         // The URL that was archived
    digest: String,   // A cryptographic hash for the HTTP response payload
    mime: String,     // The media type for the response payload
    filename: String, // the WARC file where the WARC record is located
    offset: usize,    // the byte offset for the WARC record
    length: usize,    // the length in bytes of the WARC record
    status: u16,      // the HTTP status code for the HTTP response
}

pub struct WarcTimestamp(DateTime<chrono::FixedOffset>);

#[derive(Debug)]
pub struct WarcTimestampError;

impl WarcTimestamp {
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, WarcTimestampError> {
        if let Some(warc_header_date) = record.header(WarcHeader::Date) {
            Ok(Self(
                // handle this error!
                DateTime::parse_from_rfc3339(&warc_header_date).unwrap(),
            ))
        } else {
            Err(WarcTimestampError)
        }
    }
    pub fn into_string(self) -> String {
        // Timestamp format from section 5 of the spec
        // https://specs.webrecorder.net/cdxj/0.1.0/#timestamp
        // is there an extra error to handle here?
        self.0.format("%Y%m%d%H%M%S").to_string()
    }
}

pub struct WarcUrl(Url);

#[derive(Debug)]
pub struct WarcUrlError;

impl WarcUrl {
    pub fn new(record: &Record<BufferedBody>) -> Result<Self, WarcUrlError> {
        if let Some(warc_header_url) = record.header(WarcHeader::TargetURI) {
            // propogate this error?
            Ok(Self(Url::parse(&warc_header_url).unwrap()))
        } else {
            Err(WarcUrlError)
        }
    }
    pub fn into_lowercase_string(&self) -> String {
        let url_string: String = self.0.clone().into();
        url_string.to_lowercase()
    }
    pub fn into_searchable_string(&self) -> Result<String, WarcUrlError> {
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
            Err(WarcUrlError)
        }
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
    println!("first newline is at byte {first_http_response_byte_counter}");
    println!("second newline is at byte {second_http_response_byte_counter}");

    // cut the HTTP header out of the WARC body
    // and, there is an error here to handle
    record
        .body()
        .get(first_http_response_byte_counter..second_http_response_byte_counter)
        .unwrap()
}

pub fn compose_index(warc_file_path: &Path) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    fn process_records_gzip(
        file_records: RecordIter<BufReader<MultiDecoder<BufReader<File>>>>,
        warc_file_path: &Path,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let mut record_count: u16 = 0u16;
        let mut byte_counter: u64 = 0u64;
        for record in file_records {
            record_count = record_count.wrapping_add(1);
            let unwrapped_record: Record<BufferedBody> = match record {
                Err(err) => {
                    // Any error with the record here
                    // affects the offset counter, so
                    // can't index the file!
                    panic!("Unable to index file. Record error: {err}\r\n");
                }
                Ok(record) => record,
            };
            process_record(&unwrapped_record, byte_counter, warc_file_path)?;
            // here we are getting the length of the unwrapped record header
            // plus the record body
            let record_length: u64 = unwrapped_record.content_length()
                + unwrapped_record.into_raw_parts().0.to_string().len() as u64;
            // increment the byte counter after processing the record
            byte_counter = byte_counter.wrapping_add(record_length);
        }
        println!("Total records: {record_count}");
        Ok(())
    }

    fn process_records_not_gzip(
        file_records: RecordIter<BufReader<File>>,
        warc_file_path: &Path,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let mut record_count: u16 = 0u16;
        let mut byte_counter: u64 = 0u64;
        for record in file_records {
            record_count = record_count.wrapping_add(1);
            let unwrapped_record: Record<BufferedBody> = match record {
                Err(err) => {
                    // Any error with the record here
                    // affects the offset counter, so
                    // can't index the file!
                    panic!("Unable to index file ???. Record error: {err}\r\n");
                }
                Ok(record) => record,
            };
            process_record(&unwrapped_record, byte_counter, warc_file_path)?;
            // here we are getting the length of the unwrapped record header
            // plus the record body
            let record_length: u64 = unwrapped_record.content_length()
                + unwrapped_record.into_raw_parts().0.to_string().len() as u64;
            // increment the byte counter after processing the record
            byte_counter = byte_counter.wrapping_add(record_length);
        }
        println!("Total records: {record_count}");
        Ok(())
    }

    fn process_record(
        record: &Record<BufferedBody>,
        byte_counter: u64,
        warc_file_path: &Path,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        // use something like a control flow enum to
        // organise this
        // https://doc.rust-lang.org/stable/std/ops/enum.ControlFlow.html

        // first check whether the record is either
        // a response, revisit, resource, or metadata
        let record_type: &warc::RecordType = record.warc_type();
        if [
            RecordType::Response,
            RecordType::Revisit,
            RecordType::Resource,
            RecordType::Metadata,
        ]
        .contains(record_type)
        {
            println!("\n--------");
            println!(
                "Processing record {} of type {record_type}",
                record.warc_id()
            );

            let timestamp = WarcTimestamp::new(record).unwrap();
            println!("warc timestamp is {}", timestamp.into_string());

            let record_url = WarcUrl::new(record).unwrap();
            println!("url is            {}", &record_url.into_lowercase_string());
            println!(
                "searchable url is {}",
                &record_url.into_searchable_string().unwrap()
            );

            if let Some(warc_header_url) = record.header(WarcHeader::TargetURI) {
                let json_url = &warc_header_url;
                println!("record url is {json_url}");
            } else {
                println!("No url found in record, handle this error!");
            }

            if let Some(record_digest) = record.header(WarcHeader::PayloadDigest) {
                println!("record digest is {record_digest}");
            } else {
                println!("No digest found in record, handle this error!");
            }

            println!("offset is {}", byte_counter);
            println!("length is {}", record.content_length());

            // beware! the warc content type is not the same
            // as the record content type in order to actually
            // do anything about this we need to read
            // the record body
            let mime_type: &str = if record_type == &RecordType::Revisit {
                // If the WARC record type is revisit,
                // that's the content type
                "revisit"
            } else {
                // create a list of 64 empty headers, if this is not
                // enough then you'll get a TooManyHeaders error
                let mut headers = [httparse::EMPTY_HEADER; 64];
                let header_byte_slice = cut_http_headers_from_record(record);
                // parse the raw byte array with httparse, this adds
                // data to the empty header list created above
                httparse::parse_headers(header_byte_slice, &mut headers).unwrap();
                // loop through the list of headers looking for the content-type
                let mut content_type: &str = "";
                for header in &headers {
                    if header.name == "content-type" {
                        content_type = str::from_utf8(header.value).unwrap();
                    }
                }
                content_type
            };
            println!("content type is {mime_type}");

            // Cut a slice out from the record body from
            // byte 9 to byte 12, this should be the
            // status code
            let header_byte_slice: &[u8] = &record.body()[9..12];
            // Convert it to a string, if this doesn't work
            // it'll produce unknown characters
            let header_status = String::from_utf8_lossy(header_byte_slice);
            // Parse it to a number, if there's an error it appears here!
            let header_status_int = header_status.parse::<u16>().unwrap();
            println!("header status {header_status_int}");

            let filename: String =
                if let Some(record_filename) = record.header(WarcHeader::Filename) {
                    println!("record filename is {record_filename} from file");
                    record_filename.into_owned()
                } else {
                    println!("No filename found in record, getting filename from path");
                    let filename_os_string = warc_file_path.file_name().unwrap();
                    let filename_str = filename_os_string.to_str().unwrap();
                    filename_str.to_string()
                };

            println!("filename is {filename}");

            println!("--------\n");
        }

        Ok(())
    }

    if warc_file_path.extension() == Some(OsStr::new("gz")) {
        let file_gzip: WarcReader<BufReader<MultiDecoder<BufReader<File>>>> =
            WarcReader::from_path_gzip(warc_file_path)?;
        let file_records: RecordIter<BufReader<MultiDecoder<BufReader<File>>>> =
            file_gzip.iter_records();
        process_records_gzip(file_records, warc_file_path)?;
    } else {
        let file_not_gzip: WarcReader<BufReader<File>> = WarcReader::from_path(warc_file_path)?;
        let file_records: RecordIter<BufReader<File>> = file_not_gzip.iter_records();
        process_records_not_gzip(file_records, warc_file_path)?;
    };

    Ok(())
}
