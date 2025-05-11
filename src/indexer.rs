use core::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use libflate::gzip::MultiDecoder;
use warc::{BufferedBody, Record, RecordIter, RecordType, WarcReader};

mod cdxj_index;
mod cdxj_index_errors;
use cdxj_index::*;

fn process_record(
    record: &Record<BufferedBody>,
    byte_counter: u64,
    warc_file_path: &Path,
) -> Result<CDXJIndexRecord, Box<dyn Error + Send + Sync + 'static>> {
    // use something like a control flow enum to
    // organise this
    // https://doc.rust-lang.org/stable/std/ops/enum.ControlFlow.html
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
        let parsed_record = CDXJIndexRecord {
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
        Ok(parsed_record)
    } else {
        Err(format!(
            "Record {} of type {} is not an indexable type, skipping",
            record.warc_id(),
            record.warc_type().to_string()
        )
        .into())
    }
}

pub fn compose_index(
    warc_file_path: &Path,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync + 'static>> {
    let mut record_count: usize = 0usize;
    let mut byte_counter: u64 = 0u64;
    let mut index: Vec<CDXJIndexRecord> = Vec::with_capacity(1024);

    if warc_file_path.extension() == Some(OsStr::new("gz")) {
        let file_gzip: WarcReader<BufReader<MultiDecoder<BufReader<File>>>> =
            WarcReader::from_path_gzip(warc_file_path)?;
        let file_records: RecordIter<BufReader<MultiDecoder<BufReader<File>>>> =
            file_gzip.iter_records();
        for record in file_records.enumerate() {
            record_count = record.0;
            match record.1 {
                // Need to be able to skip the record here
                // add this to a bufwriter
                Ok(record) => {
                    match process_record(&record, byte_counter, warc_file_path) {
                        Ok(processed_record) => {
                            index.push(processed_record);
                        }
                        Err(err) => eprintln!(
                            "Skipping record number {} with id {} because {err}",
                            record_count,
                            record.warc_id()
                        ),
                    }
                    // here we are getting the length of the record body
                    // in content_length, added to the length of the
                    // unwrapped record header
                    let record_length: u64 = record.content_length()
                        + record.into_raw_parts().0.to_string().len() as u64;
                    // increment the byte counter after processing the record
                    byte_counter = byte_counter.wrapping_add(record_length);
                }
                Err(err) => {
                    // Any error with the record at this
                    // point affects the offset counter,
                    // so can't index the rest of the file.
                    eprintln!("Unable to index the remainder of the file. Record error: {err}\r\n");
                    break;
                }
            }
        }
    } else {
        let file_not_gzip: WarcReader<BufReader<File>> = WarcReader::from_path(warc_file_path)?;
        let file_records: RecordIter<BufReader<File>> = file_not_gzip.iter_records();
        for record in file_records.enumerate() {
            record_count = record.0;
            match record.1 {
                // Need to be able to skip the record here
                // add this to a bufwriter
                Ok(record) => {
                    match process_record(&record, byte_counter, warc_file_path) {
                        Ok(processed_record) => {
                            index.push(processed_record);
                        }
                        Err(err) => eprintln!(
                            "Skipping record number {} with id {} because {err}",
                            record_count,
                            record.warc_id()
                        ),
                    }
                    // here we are getting the length of the record body
                    // in content_length, added to the length of the
                    // unwrapped record header
                    let record_length: u64 = record.content_length()
                        + record.into_raw_parts().0.to_string().len() as u64;
                    // increment the byte counter after processing the record
                    byte_counter = byte_counter.wrapping_add(record_length);
                }
                Err(err) => {
                    // Any error with the record at this
                    // point affects the offset counter,
                    // so can't index the rest of the file.
                    eprintln!("Unable to index the remainder of the file. Record error: {err}\r\n");
                    break;
                }
            }
        }
    };

    println!("Total records: {record_count}");

    let index = index.iter().map(|x| x.to_string());
    println!("\n\n\n index iterator is {}", index.to_string());
    Ok(index)
}
