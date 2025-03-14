use core::error::Error;
use std::path::Path;

use chrono::DateTime;
use url::Position;
use url::Url;
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

    let file = WarcReader::from_path_gzip(warc_file_path)?;

    let mut count: usize = 0;
    for record in file.iter_records() {
        // counting arithmetic is unsafe
        // do something about this in future
        count += 1;
        match record {
            Err(err) => println!("ERROR: {err}\r\n"),
            Ok(record) => {
                // use something like a control flow enum to
                // organise this
                // https://doc.rust-lang.org/stable/std/ops/enum.ControlFlow.html

                // Compose searchable url from WARC Header
                if let Some(url) = record.header(WarcHeader::TargetURI) {
                    let searchable_url = create_searchable_url(&url)?;
                    println!("{searchable_url}");
                } else {
                    println!("No url found in record, handle this error!");
                }

                // Compose timestamp from WARC header
                if let Some(date) = record.header(WarcHeader::Date) {
                    let parsed_datetime = DateTime::parse_from_rfc3339(&date)?;
                    // Timestamp format from section 5 of the spec
                    // https://specs.webrecorder.net/cdxj/0.1.0/#timestamp
                    let timestamp = format!("{}", parsed_datetime.format("%Y%m%d%H%M%S"));
                    println!("{timestamp}");
                } else {
                    println!("No date found in record, handle this error!");
                }
            }
        }
    }

    println!("Total records: {count}");

    Ok(())
}
