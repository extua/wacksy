//! Reads WARC files and wraps them up into a WACZ archive.
//! examples todo.

pub mod datapackage;
pub mod indexer;
use rawzip::{CompressionMethod, Error, ZipArchiveWriter, ZipDataWriter};

/// Set the WACZ version of the file being created,
/// deprecated in WACZ 1.2.0.
pub const WACZ_VERSION: &str = "1.1.1";

/// This struct contains various resources as
/// byte arrays, ready to be zipped.
pub struct Wacz {
    pub warc_file: Vec<u8>,
    pub data_package_bytes: Vec<u8>,
    pub data_package_digest_bytes: Vec<u8>,
    pub cdxj_index_bytes: Vec<u8>,
    pub pages_index_bytes: Vec<u8>,
}
impl Wacz {
    fn add_file_to_archive(
        archive: &mut ZipArchiveWriter<&mut Vec<u8>>,
        compression_method: CompressionMethod,
        file_data: &[u8],
        file_path: &str,
    ) {
        // Start a new file in our zip archive.
        let mut file = archive
            .new_file(file_path)
            .compression_method(compression_method)
            .create()
            .unwrap();

        // Wrap the file in a ZipDataWriter, which will track information for the
        // Zip data descriptor (like uncompressed size and crc).
        let mut writer = ZipDataWriter::new(&mut file);

        // Copy the data to the writer.
        std::io::copy(&mut &*file_data, &mut writer).unwrap();

        // Finish the file, which will return the finalized data descriptor
        let (_, descriptor) = writer.finish().unwrap();

        let uncompressed_size = descriptor.uncompressed_size();

        println!("wrote {uncompressed_size} bytes to {file_path}");

        // Write out the data descriptor and return the number of bytes the data compressed to.
        file.finish(descriptor).unwrap();
    }

    /// # Zipper
    ///
    /// This function should accept a... struct, with a warc file,
    /// which is a stream of bytes, and some other things, also streams of bytes.
    ///
    /// # Errors
    ///
    /// Will return a rawzip error if anything goes wrong with adding files
    /// files to the archive.
    pub fn zip_dir(wacz_object: &Self) -> Result<Vec<u8>, Error> {
        // Create a new Zip archive in memory.
        let mut output = Vec::new();
        let mut archive = ZipArchiveWriter::new(&mut output);

        // Set compression method to Store (no compression).
        let compression_method = CompressionMethod::Store;

        // this should be an iterator?
        // iterate over everything listed in the datapackage!
        // and add it recursively
        // and *then*, add the datapackage digest at the end
        Self::add_file_to_archive(
            &mut archive,
            compression_method,
            &wacz_object.warc_file,
            // at this point if the file is gzipped, the file should be 'data.gz'
            "archive/data.warc",
        );
        Self::add_file_to_archive(
            &mut archive,
            compression_method,
            &wacz_object.data_package_bytes,
            "datapackage.json",
        );
        Self::add_file_to_archive(
            &mut archive,
            compression_method,
            &wacz_object.data_package_digest_bytes,
            "datapackage-digest.json",
        );
        Self::add_file_to_archive(
            &mut archive,
            compression_method,
            &wacz_object.cdxj_index_bytes,
            "indexes/index.cdxj",
        );
        Self::add_file_to_archive(
            &mut archive,
            compression_method,
            &wacz_object.pages_index_bytes,
            "pages/pages.jsonl",
        );

        // Finish the archive, which will write the central directory.
        archive.finish()?;

        return Ok(output);
    }
}
