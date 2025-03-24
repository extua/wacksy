use core::error::Error;
use core::str;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::{ffi::OsStr, io::Read};

use chrono::DateTime;
use libflate::gzip::MultiDecoder;
use url::{Position, Url};
use warc::{BufferedBody, Record, RecordIter, RecordType, WarcHeader, WarcReader};

pub fn compose_index(warc_file_path: &Path) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    fn create_searchable_url(url: &str) -> Result<String, Box<dyn Error + Send + Sync + 'static>> {
        let lowercase_url = url.to_lowercase();
        let parsed_url = Url::parse(&lowercase_url);
        match parsed_url {
            Err(err) => return Err(format!("Error parsing URL: {err}\r\n").into()),
            Ok(successfully_parsed_url) => {
                if let Some(host) = successfully_parsed_url.host_str() {
                    // split the host string into an array at each dot
                    let mut host_split: Vec<&str> = host.split('.').collect();
                    // reverse the order of the array
                    host_split.reverse();
                    // join the array back into a comma-separated string
                    let host_reversed = host_split.join(",");
                    // capture everything else on the end of the url
                    let url_path = &successfully_parsed_url[Position::BeforePath..];
                    // put it all together
                    let searchable_url = format!("{host_reversed}){url_path}");
                    return Ok(searchable_url);
                }
                return Err(format!(
                    "No hostname found in {lowercase_url}, handle this error!\r\n"
                )
                .into());
            }
        }
    }

    if warc_file_path.extension() == Some(OsStr::new("gz")) {
        let file_gzip: WarcReader<BufReader<MultiDecoder<BufReader<File>>>> =
            WarcReader::from_path_gzip(warc_file_path)?;
        let file_records: RecordIter<BufReader<MultiDecoder<BufReader<File>>>> =
            file_gzip.iter_records();
        process_records_gzip(file_records)?;
    } else {
        let file_not_gzip: WarcReader<BufReader<File>> = WarcReader::from_path(warc_file_path)?;
        let file_records: RecordIter<BufReader<File>> = file_not_gzip.iter_records();
        process_records_not_gzip(file_records)?;
    };

    // struct CDXJIndexObject {
    //     url: Url,         // The URL that was archived
    //     digest: String,   // A cryptographic hash for the HTTP response payload
    //     mime: String,     // The media type for the response payload
    //     filename: String, // the WARC file where the WARC record is located
    //     offset: usize,    // the byte offset for the WARC record
    //     length: String,   // the length in bytes of the WARC record
    //     status: String,   // the HTTP status code for the HTTP response
    // }

    fn process_records_gzip(
        file_records: RecordIter<BufReader<MultiDecoder<BufReader<File>>>>,
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
            process_record(&unwrapped_record, &byte_counter)?;
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
            process_record(&unwrapped_record, &byte_counter)?;
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
        byte_counter: &u64,
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
        .contains(&record_type)
        {
            println!("\n--------");
            println!(
                "Processing record {} of type {record_type}",
                record.warc_id()
            );

            // Compose searchable url from WARC Header
            if let Some(warc_header_url) = record.header(WarcHeader::TargetURI) {
                let searchable_url = create_searchable_url(&warc_header_url)?;
                println!("searchable url is {searchable_url}");
            } else {
                println!("No url found in record, handle this error!");
            }

            // Compose timestamp from WARC header
            if let Some(warc_header_date) = record.header(WarcHeader::Date) {
                let parsed_datetime = DateTime::parse_from_rfc3339(&warc_header_date)?;
                // Timestamp format from section 5 of the spec
                // https://specs.webrecorder.net/cdxj/0.1.0/#timestamp
                let timestamp = format!("{}", parsed_datetime.format("%Y%m%d%H%M%S"));
                println!("timestamp is {timestamp}");
            } else {
                println!("No date found in record, handle this error!");
            }

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

            // beware! the warc content type is not the same
            // as the record content type
            // in order to actually do anything about this we need
            // to read the record body
            // let record_content_type = record.body();

            println!("offset is {}", byte_counter);
            println!("length is {}", record.content_length());

            let mime_type: &str = if record_type == &RecordType::Revisit {
                "revisit"
            } else {
                println!("parse this from http header");

                let record_body: &[u8] = record.body();

                // Find the position of the first newline, this will
                // get just the headers, not the full request, see
                // https://stackoverflow.com/questions/69610022/how-can-i-get-httparse-to-parse-my-request-correctly
                let mut first_http_response_byte_counter: usize = 0;
                for byte in record_body {
                    if byte == &0xA {
                        first_http_response_byte_counter += 1;
                        break;
                    } else {
                        first_http_response_byte_counter += 1;
                        continue;
                    }
                }

                // Find the position of the first sequence of
                // two newlines, this ends the HTTP 1.1 header block
                // according to section 3 of RFC7230
                let mut second_http_response_byte_counter: usize = 0;
                for byte in record_body {
                    let next_byte: &u8 = record_body.iter().next().unwrap();
                    if byte == &0xA && next_byte == &0xA {
                        break;
                    } else {
                        second_http_response_byte_counter += 1;
                        continue;
                    }
                }
                println!("first newline is at byte {first_http_response_byte_counter}");
                println!("second newline is at byte {second_http_response_byte_counter}");

                // cut the HTTP header out of the WARC body
                let header_byte_slice = &record_body
                    [first_http_response_byte_counter..second_http_response_byte_counter];

                // create a list of 50 empty headers, if this is not enough then
                // you'll get a TooManyHeaders error
                let mut headers = [httparse::EMPTY_HEADER; 50];
                // parse the raw byte array with httparse
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

            println!("--------\n");

            // let record_content_type = &record.body();

            // let record_filename: &str = &record.header(WarcHeader::Filename).unwrap();
            // println!("record filename is {record_filename}");

            // if let Some(record_filename) = record.header(WarcHeader::Filename) {
            //     let json_filename = &record_filename;
            //     println!("record filename is {json_filename}");
            // } else {
            //     println!("No filename found in record, handle this error!");
            // }
        }

        Ok(())
    }

    Ok(())
}
