use core::error::Error;
use std::{fs, path::Path};
use wacksy::{DataPackage, Wacz, compose_datapackage, indexer::CDXJIndex, zip_dir};

fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let warc_file_path: &Path = Path::new("examples/rec-e7e68da067d0-20250423121042981-0.warc.gz");

    let warc_file = fs::read(warc_file_path)?;

    let index = CDXJIndex::new(warc_file_path)?;

    let index_bytes = index.to_string().into_bytes();

    let data_package = compose_datapackage(&warc_file, &index_bytes);
    let data_package_digest = DataPackage::digest(&data_package)?;

    let data_package_digest_bytes = serde_json::to_vec(&data_package_digest)?;
    let data_package_bytes = serde_json::to_vec(&data_package)?;

    let wacz_object: Wacz = {
        Wacz {
            warc_file,
            data_package_bytes,
            data_package_digest_bytes,
            index_bytes,
        }
    };

    // This needs to be parsed into a file!
    let wacz_data: Vec<u8> = zip_dir(&wacz_object).unwrap();
    let path: &Path = Path::new("wacz_example.wacz");
    // fs::write(path, wacz_data)?;
    Ok(())
}
