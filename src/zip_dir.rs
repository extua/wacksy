use rawzip::{CompressionMethod, RawZipWriter, ZipArchiveWriter, ZipEntryOptions};
use std::{fs::File, io::Write};

pub fn zip_dir(wacz_file: File) {
    let mut writer = ZipArchiveWriter::new(wacz_file);
    writer.new_dir("archive/").unwrap();
    writer.new_dir("indexes/").unwrap();
    writer.new_dir("pages/").unwrap();

    let options = ZipEntryOptions::default().compression_method(CompressionMethod::Store);

    let mut warc_file = writer.new_file("archive/data.warc", options).unwrap();

    let output = {
        let mut warc_file_writer = RawZipWriter::new(&mut warc_file);

        warc_file_writer.write_all(b"Hello, world!").unwrap();
        let (_, output) = warc_file_writer.finish().unwrap();
        output
    };

    warc_file.finish(output).unwrap();
    writer.finish().unwrap();
}
