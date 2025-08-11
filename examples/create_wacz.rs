use std::error::Error;
use std::{fs, path::Path};
use wacksy::{Wacz, datapackage::DataPackage, indexer::Index};

fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let warc_file_path: &Path = Path::new("examples/rec-e7e68da067d0-20250423121042981-0.warc.gz");

    let warc_file = fs::read(warc_file_path)?;

    let index = Index::index_file(warc_file_path)?;
    println!("Read {} records", index.2);

    let cdxj_index_bytes = index.0.to_string().into_bytes();
    let pages_index_bytes = index.1.to_string().into_bytes();

    let data_package = DataPackage::new(&warc_file, &index)?;
    let data_package_digest = DataPackage::digest(&data_package)?;

    let data_package_digest_bytes = serde_json::to_vec(&data_package_digest)?;
    let data_package_bytes = serde_json::to_vec(&data_package)?;

    let wacz_object: Wacz = {
        Wacz {
            warc_file,
            data_package_bytes,
            data_package_digest_bytes,
            cdxj_index_bytes,
            pages_index_bytes,
        }
    };

    // This needs to be parsed into a file!
    let wacz_data: Vec<u8> = Wacz::zip_dir(&wacz_object)?;
    let path: &Path = Path::new("wacz_example.wacz");
    fs::write(path, wacz_data)?;
    Ok(())
}
