use core::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use chrono::DateTime;
use libflate::gzip::MultiDecoder;
use url::Position;
use url::Url;
use warc::BufferedBody;
use warc::Record;
use warc::WarcHeader;
use warc::WarcReader;

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
        process_records_gzip(file_gzip)?;
    } else {
        let file_not_gzip: WarcReader<BufReader<File>> = WarcReader::from_path(warc_file_path)?;
        process_records_not_gzip(file_not_gzip)?;
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
        file_gzip: WarcReader<BufReader<MultiDecoder<BufReader<File>>>>,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let mut count: usize = 0;
        let file_records = file_gzip.iter_records();
        for record in file_records {
            count = count.wrapping_add(1);
            let unwrapped_record = match record {
                Err(err) => {
                    // better error handling here!
                    println!("Record error: {err}\r\n");
                    continue;
                }
                Ok(record) => record,
            };
            process_record(unwrapped_record)?;
        }
        println!("Total records: {count}");
        Ok(())
    }

    fn process_records_not_gzip(
        file_not_gzip: WarcReader<BufReader<File>>,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        let mut count: usize = 0;
        let file_records = file_not_gzip.iter_records();
        for record in file_records {
            count = count.wrapping_add(1);
            let unwrapped_record = match record {
                Err(err) => {
                    // better error handling here!
                    println!("Record error: {err}\r\n");
                    continue;
                }
                Ok(record) => record,
            };
            process_record(unwrapped_record)?;
        }
        println!("Total records: {count}");
        Ok(())
    }

    fn process_record(
        record: Record<BufferedBody>,
    ) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
        // use something like a control flow enum to
        // organise this
        // https://doc.rust-lang.org/stable/std/ops/enum.ControlFlow.html

        // println!("{record}");

        // first check whether the record is a response
        let record_type: &str = &record.header(WarcHeader::WarcType).unwrap();
        if ["response", "revisit", "resource", "metadata"].contains(&record_type) {
            println!("This record is a {record_type}");

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
            let record_content_type: &str = &record.header(WarcHeader::ContentType).unwrap();
            if let Some(record_mime_type) = record_content_type.split(";").into_iter().nth(0) {
                println!("record content type is {record_mime_type}");
            } else {
                println!("Unable to read the MIME type, handle this error!");
            }

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
