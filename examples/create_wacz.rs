use std::{
    fs,
    path::Path,
};
use wacksy::{create_datapackage, zip_dir, DataPackage, Wacz};

fn main() {
    let warc_file = fs::read("examples/warc_example.warc").unwrap();
    let data_package = create_datapackage(&warc_file);
    let data_package_digest = DataPackage::create_digest(&data_package);
    let data_package_digest_bytes = serde_json::to_vec(&data_package_digest).unwrap();


    let wacz_object: Wacz = {
        Wacz {
            warc_file,
            data_package,
            data_package_digest_bytes,
        }
    };

    // This needs to be parsed into a file!
    let wacz_data: Vec<u8> = zip_dir(&wacz_object).unwrap();
    let path: &Path = Path::new("wacz_example.wacz");
    fs::write(path, wacz_data).unwrap();
}
