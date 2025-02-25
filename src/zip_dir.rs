use rawzip::{CompressionMethod, Error, RawZipWriter, ZipArchiveWriter, ZipEntryOptions};

pub fn zip_dir(warc_file: &[u8]) -> Result<Vec<u8>, Error> {
    // Create a new Zip archive in memory.
    let mut output = Vec::new();
    let mut archive = ZipArchiveWriter::new(&mut output);

    // Set options, with no compression.
    let options = ZipEntryOptions::default().compression_method(CompressionMethod::Store);

    // Start a new file in our zip archive.
    let mut file = archive.new_file("archive/data.warc", options).unwrap();

    // Wrap the file in a RawZipWriter, which will track information for the
    // Zip data descriptor (like uncompressed size and crc).
    let mut writer = RawZipWriter::new(&mut file);

    // Copy the data to the writer.
    std::io::copy(&mut &warc_file[..], &mut writer).unwrap();

    let descriptor = {
        // Finish the file, which will return the finalized data descriptor
        let (_, descriptor) = writer.finish().unwrap();
        descriptor
    };

    // Write out the data descriptor and return the number of bytes the data compressed to.
    let compressed = file.finish(descriptor)?;

    println!("wrote {compressed} bytes");

    // Finish the archive, which will write the central directory.
    archive.finish()?;

    Ok(output)
}
