use std::borrow::Borrow;
use std::error::Error;
use std::path::Path;

use chrono::format::parse;
use url::Url;
// use warc::BufferedBody;
// use warc::Record;
// use warc::RawRecordHeader;
use warc::WarcHeader;
use warc::WarcReader;

pub fn compose_index(warc_file_path: &Path) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let file = WarcReader::from_path(warc_file_path)?;

    let mut count = 0;
    for record in file.iter_records() {
        count += 1;
        match record {
            Err(err) => println!("ERROR: {}\r\n", err),
            Ok(record) => {
                if let Some(url) = record.header(WarcHeader::TargetURI) {
                    // Cow to String
                    let lowercase_url = url.as_ref().to_lowercase();
                    let parsed_url = Url::parse(&lowercase_url);
                    match parsed_url {
                        Err(err) => println!("Error parsing URL: {}\r\n", err),
                        Ok(parsed_url) => {
                            if let Some(host) = parsed_url.host_str() {
                                println!("{host}");
                            } else {
                                println!(
                                    "No hostname found in {lowercase_url}, handle this error!"
                                );
                            }
                        }
                    };
                } else {
                    println!("No url found in record, handle this error!");
                }
            }
        }
    }

    // println!("{record:?}");
    println!("Total records: {}", count);

    Ok(())
}
