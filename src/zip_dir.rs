use rawzip::{CompressionMethod, Error, RawZipWriter, ZipArchiveWriter, ZipEntryOptions};

pub fn zip_dir(warc_file: &[u8]) -> Result<Vec<u8>, Error> {
    // Create a new Zip archive in memory.
    let mut output = Vec::new();
    let mut archive = ZipArchiveWriter::new(&mut output);

    // Set options, with no compression.
    let options = ZipEntryOptions::default().compression_method(CompressionMethod::Store);

    fn add_file_to_archive(
        archive: &mut ZipArchiveWriter<&mut Vec<u8>>,
        options: ZipEntryOptions,
        file_data: &[u8],
        file_path: &str,
    ) {
        // Start a new file in our zip archive.
        let mut file = archive.new_file(file_path, options).unwrap();

        // Wrap the file in a RawZipWriter, which will track information for the
        // Zip data descriptor (like uncompressed size and crc).
        let mut writer = RawZipWriter::new(&mut file);

        // Copy the data to the writer.
        std::io::copy(&mut &file_data[..], &mut writer).unwrap();

        // Finish the file, which will return the finalized data descriptor
        let (_, descriptor) = writer.finish().unwrap();

        let uncompressed_size = descriptor.uncompressed_size();

        println!("wrote {uncompressed_size} bytes to {file_path}");

        // Write out the data descriptor and return the number of bytes the data compressed to.
        file.finish(descriptor).unwrap();
    }

    let file_path: &str = "archive/data.warc";

    add_file_to_archive(&mut archive, options, warc_file, file_path);

    // Finish the archive, which will write the central directory.
    archive.finish()?;

    Ok(output)
}
