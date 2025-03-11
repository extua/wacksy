use std::error::Error;
use std::path::Path;

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
                println!("{}: {}", WarcHeader::RecordID, record.warc_id(),);
                println!("{}: {}", WarcHeader::Date, record.date(),);
                println!();
            }
        }
    }

    println!("Total records: {}", count);

    Ok(())
}
