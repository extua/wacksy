use std::error::Error;
use std::fs::File;
use std::path::Path;

use warc::WarcHeader;
use warc::WarcReader;

pub fn compose_index(warc_file_path: &Path) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let file: WarcReader<std::io::BufReader<std::fs::File>> =
        WarcReader::from_path("examples/warc_example.warc")?;

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
