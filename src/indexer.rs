use core::error::Error;
use std::ffi::OsStr;
use std::path::Path;

use chrono::DateTime;
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

    // let file = WarcReader::from_path_gzip(warc_file_path)?;

    if warc_file_path.extension() == Some(OsStr::new("gz")) {
        let file_gzip: WarcReader<
            std::io::BufReader<libflate::gzip::MultiDecoder<std::io::BufReader<std::fs::File>>>,
        > = WarcReader::from_path_gzip(warc_file_path)?;
        process_records_gzip(file_gzip);
    } else {
        let file_not_gzip: WarcReader<std::io::BufReader<std::fs::File>> =
            WarcReader::from_path(warc_file_path)?;
        process_records_not_gzip(file_not_gzip);
    };

    // let file2 = WarcReader::from_path(warc_file_path)?;

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
        file_gzip: WarcReader<
            std::io::BufReader<libflate::gzip::MultiDecoder<std::io::BufReader<std::fs::File>>>,
        >,
    ) {
        let mut count: usize = 0;
        let file_records = file_gzip.iter_records();
        for record in file_records {
            // error handlnig here!
            let unwrapped_record = record.unwrap();
            process_record(unwrapped_record, count);
        }
        println!("Total records: {count}");
    }

    fn process_records_not_gzip(file_not_gzip: WarcReader<std::io::BufReader<std::fs::File>>) {
        let mut count: usize = 0;
        let file_records = file_not_gzip.iter_records();
        for record in file_records {
            // error handlnig here!
            let unwrapped_record = record.unwrap();
            process_record(unwrapped_record, count);
        }
        println!("Total records: {count}");
    }

    fn process_record(record: Record<BufferedBody>, mut count: usize) {
        // counting arithmetic is unsafe
        // do something about this in future
        count = count.wrapping_add(1);

        // use something like a control flow enum to
        // organise this
        // https://doc.rust-lang.org/stable/std/ops/enum.ControlFlow.html

        // println!("{record:?}");

        // Compose searchable url from WARC Header
        if let Some(warc_header_url) = record.header(WarcHeader::TargetURI) {
            let searchable_url = create_searchable_url(&warc_header_url).unwrap();
            println!("{searchable_url}");
        } else {
            println!("No url found in record, handle this error!");
        }

        // Compose timestamp from WARC header
        if let Some(warc_header_date) = record.header(WarcHeader::Date) {
            let parsed_datetime = DateTime::parse_from_rfc3339(&warc_header_date).unwrap();
            // Timestamp format from section 5 of the spec
            // https://specs.webrecorder.net/cdxj/0.1.0/#timestamp
            let timestamp = format!("{}", parsed_datetime.format("%Y%m%d%H%M%S"));
            println!("{timestamp}");
        } else {
            println!("No date found in record, handle this error!");
        }

        if let Some(warc_header_url) = record.header(WarcHeader::TargetURI) {
            let json_url = &warc_header_url;
            println!("{json_url}");
        } else {
            println!("No url found in record, handle this error!");
        }

        // let record_digest: &str = &record.header(WarcHeader::PayloadDigest).unwrap();

        // // this isn't quite the mime type, needs some
        // // processing to remove the rest of the content
        // let record_mime_type: &str = &record.header(WarcHeader::ContentType).unwrap();

        // println!("{record_mime_type}");
    }

    Ok(())
}
