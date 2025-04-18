use core::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use libflate::gzip::MultiDecoder;
use warc::{BufferedBody, Record, RecordIter, RecordType, WarcReader};

mod cdxj_index;
use cdxj_index::*;

pub fn compose_index(
    warc_file_path: &Path,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync + 'static>> {
    fn process_records_gzip(
        file_records: RecordIter<BufReader<MultiDecoder<BufReader<File>>>>,
        warc_file_path: &Path,
    ) -> Result<Vec<u8>, Box<dyn Error + Send + Sync + 'static>> {
        let mut record_count: u16 = 0u16;
        let mut byte_counter: u64 = 0u64;
        let mut index: Vec<u8> = Vec::with_capacity(128);
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
            // Need to be able to skip the record here
            // add this to a bufwriter
            if let Ok(processed_record) =
                process_record(&unwrapped_record, byte_counter, warc_file_path)
            {
                let record_some = processed_record;
                let record_bytes = record_some.as_bytes();
                index.extend_from_slice(record_bytes);
            } else {
                eprintln!("unable to process record, skipping");
            }

            // here we are getting the length of the unwrapped record header
            // plus the record body
            let record_length: u64 = unwrapped_record.content_length()
                + unwrapped_record.into_raw_parts().0.to_string().len() as u64;
            // increment the byte counter after processing the record
            byte_counter = byte_counter.wrapping_add(record_length);
        }
        println!("Total records: {record_count}");
        Ok(index)
    }

    fn process_records_not_gzip(
        file_records: RecordIter<BufReader<File>>,
        warc_file_path: &Path,
    ) -> Result<Vec<u8>, Box<dyn Error + Send + Sync + 'static>> {
        let mut record_count: u16 = 0u16;
        let mut byte_counter: u64 = 0u64;
        let mut index: Vec<u8> = Vec::with_capacity(128);
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
            // Need to be able to skip the record here
            // add this to a bufwriter
            match process_record(&unwrapped_record, byte_counter, warc_file_path) {
                Ok(processed_record) => {
                    let record_some = processed_record;
                    let record_bytes = record_some.as_bytes();
                    index.extend_from_slice(record_bytes);
                }
                Err(err) => eprintln!(
                    "Skipping record {} because {err}",
                    unwrapped_record.warc_id()
                ),
            }

            // here we are getting the length of the unwrapped record header
            // plus the record body
            let record_length: u64 = unwrapped_record.content_length()
                + unwrapped_record.into_raw_parts().0.to_string().len() as u64;
            // increment the byte counter after processing the record
            byte_counter = byte_counter.wrapping_add(record_length);
        }
        println!("Total records: {record_count}");
        Ok(index)
    }

    fn process_record(
        record: &Record<BufferedBody>,
        byte_counter: u64,
        warc_file_path: &Path,
    ) -> Result<String, Box<dyn Error + Send + Sync + 'static>> {
        // use something like a control flow enum to
        // organise this
        // https://doc.rust-lang.org/stable/std/ops/enum.ControlFlow.html
        let timestamp = RecordTimestamp::new(record).unwrap();
        let url = RecordUrl::new(record)?;
        let digest = RecordDigest::new(record).unwrap();
        let searchable_url = url.as_searchable_string().unwrap();
        let mime = RecordContentType::new(record)?;
        let status = RecordStatus::new(record)?;
        let filename = WarcFilename::new(record, warc_file_path).unwrap();

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
            Ok(parsed_record.to_string())
        } else {
            Err("Record with ID HERE is not of an indexable type, skipping".into())
        }
    }

    let index = if warc_file_path.extension() == Some(OsStr::new("gz")) {
        let file_gzip: WarcReader<BufReader<MultiDecoder<BufReader<File>>>> =
            WarcReader::from_path_gzip(warc_file_path)?;
        let file_records: RecordIter<BufReader<MultiDecoder<BufReader<File>>>> =
            file_gzip.iter_records();
        process_records_gzip(file_records, warc_file_path)?
    } else {
        let file_not_gzip: WarcReader<BufReader<File>> = WarcReader::from_path(warc_file_path)?;
        let file_records: RecordIter<BufReader<File>> = file_not_gzip.iter_records();
        process_records_not_gzip(file_records, warc_file_path)?
    };

    Ok(index)
}
