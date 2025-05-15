use crate::Wacz;
use rawzip::{CompressionMethod, Error, ZipArchiveWriter, ZipDataWriter, ZipEntryOptions};

/// # Zipper
/// 
/// this function should accept a... struct, with a warc file,
/// which is a stream of bytes, and some other things, also streams of bytes
/// 
/// # Errors
///
/// Will return a rawzip error if anything goes wrong with adding files
/// files to the archive.
pub fn zip_dir(wacz_object: &Wacz) -> Result<Vec<u8>, Error> {
    fn add_file_to_archive(
        archive: &mut ZipArchiveWriter<&mut Vec<u8>>,
        options: ZipEntryOptions,
        file_data: &[u8],
        file_path: &str,
    ) {
        // Start a new file in our zip archive.
        let mut file = archive.new_file(file_path, options).unwrap();

        // Wrap the file in a ZipDataWriter, which will track information for the
        // Zip data descriptor (like uncompressed size and crc).
        let mut writer = ZipDataWriter::new(&mut file);

        // Copy the data to the writer.
        std::io::copy(&mut &file_data[..], &mut writer).unwrap();

        // Finish the file, which will return the finalized data descriptor
        let (_, descriptor) = writer.finish().unwrap();

        let uncompressed_size = descriptor.uncompressed_size();

        println!("wrote {uncompressed_size} bytes to {file_path}");

        // Write out the data descriptor and return the number of bytes the data compressed to.
        file.finish(descriptor).unwrap();
    }

    // Create a new Zip archive in memory.
    let mut output = Vec::new();
    let mut archive = ZipArchiveWriter::new(&mut output);

    // Set options, with no compression.
    let options = ZipEntryOptions::default().compression_method(CompressionMethod::Store);

    // this should be an iterator?
    // iterate over everything in the struct and add it recursively
    add_file_to_archive(
        &mut archive,
        options,
        &wacz_object.warc_file,
        // at this point if the file is gzipped, the file should be 'data.gz'
        "archive/data.warc",
    );
    add_file_to_archive(
        &mut archive,
        options,
        &wacz_object.data_package_bytes,
        "datapackage.json",
    );
    add_file_to_archive(
        &mut archive,
        options,
        &wacz_object.data_package_digest_bytes,
        "datapackage-digest.json",
    );
    add_file_to_archive(
        &mut archive,
        options,
        &wacz_object.index_bytes,
        "indexes/index.cdxj",
    );

    // Finish the archive, which will write the central directory.
    archive.finish()?;

    return Ok(output)
}
