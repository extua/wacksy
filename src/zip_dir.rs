use crate::Wacz;
use rawzip::{CompressionMethod, Error, RawZipWriter, ZipArchiveWriter, ZipEntryOptions};

// this function should accept a... struct, with a warc file,
// which is a stream of bytes, and some other things, also streams of bytes
pub fn zip_dir(wacz_object: &Wacz) -> Result<Vec<u8>, Error> {
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

    // this should be an iterator?
    // iterate over everything in the struct and add it recursively
    add_file_to_archive(&mut archive, options, &wacz_object.warc_file, file_path);

    // Finish the archive, which will write the central directory.
    archive.finish()?;

    Ok(output)
}
