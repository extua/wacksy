//! Reads WARC files and wraps them up into a WACZ archive.
//! examples todo.

pub mod datapackage;
pub mod indexer;
use std::path::Path;

use rawzip::{CompressionMethod, ZipArchiveWriter, ZipDataWriter};

use crate::{
    datapackage::{DataPackage, DataPackageDigest},
    indexer::{CDXJIndex, Index, PageIndex},
};

/// Set the WACZ version of the file being created,
/// deprecated in WACZ 1.2.0.
pub const WACZ_VERSION: &str = "1.1.1";

/// This struct contains various resources as
/// byte arrays, ready to be zipped.
pub struct WACZ {
    pub datapackage: DataPackage,
    pub datapackage_digest: DataPackageDigest,
    pub cdxj_index: CDXJIndex,
    pub pages_index: PageIndex,
}
impl WACZ {
    #[must_use]
    pub fn from_file(warc_file_path: &Path) -> Self {
        let index = Index::index_file(warc_file_path).unwrap();
        let datapackage = DataPackage::new(warc_file_path, &index).unwrap();
        let datapackage_digest = datapackage.digest().unwrap();

        return Self {
            datapackage,
            datapackage_digest,
            cdxj_index: index.cdxj,
            pages_index: index.pages,
        };
    }

    /// # Zipper
    ///
    /// This function should accept a WACZ struct.
    /// explain what else
    ///
    /// # Errors
    ///
    /// Will return a rawzip error if anything goes wrong with adding files
    /// files to the archive.
    pub fn zip(&self) -> Result<Vec<u8>, rawzip::Error> {
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

        // Create a new Zip archive in memory.
        let mut output = Vec::new();
        let mut archive = ZipArchiveWriter::new(&mut output);

        // Set compression method to Store (no compression).
        let compression_method = CompressionMethod::Store;

        // iterate over every resource in the datapackage
        for datapackage_resource in &self.datapackage.resources {
            add_file_to_archive(
                &mut archive,
                compression_method,
                &datapackage_resource.content,
                &datapackage_resource.path,
            );
        }

        // add datapackage file
        add_file_to_archive(
            &mut archive,
            compression_method,
            &serde_json::to_vec(&self.datapackage).unwrap(),
            "datapackage.json",
        );

        // add digest file
        add_file_to_archive(
            &mut archive,
            compression_method,
            &serde_json::to_vec(&self.datapackage_digest).unwrap(),
            "datapackage-digest.json",
        );

        // Finish the archive, which will write the central directory.
        archive.finish()?;

        return Ok(output);
    }
}
