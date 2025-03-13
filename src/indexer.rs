use std::borrow::Cow;
use std::error::Error;
use std::path::Path;

use url::Position;
use url::Url;
use warc::WarcHeader;
use warc::WarcReader;

fn create_searchable_url(
    url: Cow<'_, str>,
) -> Result<String, Box<dyn Error + Send + Sync + 'static>> {
    // Cow to String
    let lowercase_url = url.as_ref().to_lowercase();
    let parsed_url = Url::parse(&lowercase_url);
    match parsed_url {
        Err(err) => Err(format!("Error parsing URL: {err}\r\n").into()),
        Ok(parsed_url) => {
            if let Some(host) = parsed_url.host_str() {
                let mut host_split: Vec<&str> = host.split('.').collect();
                host_split.reverse();
                let host_reversed = host_split.join(",");
                let url_path = &parsed_url[Position::BeforePath..];
                let searchable_url = format!("{host_reversed}){url_path}");
                Ok(searchable_url)
            } else {
                Err(format!("No hostname found in {lowercase_url}, handle this error!\r\n").into())
            }
        }
    }
}

pub fn compose_index(warc_file_path: &Path) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let file = WarcReader::from_path_gzip(warc_file_path)?;

    let mut count = 0;
    for record in file.iter_records() {
        count += 1;
        match record {
            Err(err) => println!("ERROR: {}\r\n", err),
            Ok(record) => {
                if let Some(url) = record.header(WarcHeader::TargetURI) {
                    let searchable_url = create_searchable_url(url)?;
                    println!("{searchable_url}");
                } else {
                    println!("No url found in record, handle this error!");
                }
            }
        }
    }

    println!("Total records: {count}");

    Ok(())
}
