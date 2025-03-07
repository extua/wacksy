use std::{fs, path::Path};
use wacksy::{DataPackage, Wacz, compose_datapackage, zip_dir};

fn main() {
    let warc_file = fs::read("examples/warc_example.warc").unwrap();
    let data_package = compose_datapackage(&warc_file);
    let data_package_digest = DataPackage::digest(&data_package);

    let data_package_digest_bytes = serde_json::to_vec(&data_package_digest).unwrap();
    let data_package_bytes = serde_json::to_vec(&data_package).unwrap();

    let wacz_object: Wacz = {
        Wacz {
            warc_file,
            data_package_bytes,
            data_package_digest_bytes,
        }
    };

    // This needs to be parsed into a file!
    let wacz_data: Vec<u8> = zip_dir(&wacz_object).unwrap();
    let path: &Path = Path::new("wacz_example.wacz");
    fs::write(path, wacz_data).unwrap();
}
