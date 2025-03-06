use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::WACZ_VERSION;

// Link to the spec
// https://specs.webrecorder.net/wacz/1.1.1/#datapackage-json

#[derive(Serialize, Deserialize)]
pub struct DataPackage {
    pub profile: String,
    pub wacz_version: String,
    pub resources: Vec<DataPackageResource>,
}

#[derive(Serialize, Deserialize)]
pub struct DataPackageResource {
    pub name: String,
    pub path: String,
    pub hash: String,
    pub bytes: usize,
}

#[derive(Serialize, Deserialize)]
pub struct DataPackageDigest {
    pub path: String,
    pub hash: String,
}

impl DataPackage {
    pub fn new(resources: Vec<DataPackageResource>) -> Self {
        DataPackage {
            profile: "data-package".to_owned(),
            wacz_version: WACZ_VERSION.to_owned(),
            resources,
        }
    }
    pub fn create_resource(path: &Path, file_bytes: Vec<u8>) -> DataPackageResource {
        // handle the option-result, but there's not
        // much to be done about this unfortunately
        let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let path = path.to_str().unwrap().to_owned();

        // create a sha256 hash, from documentation
        // here https://docs.rs/sha2/latest/sha2/
        // create a Sha256 object
        let file_hash = Sha256::digest(&file_bytes);
        let file_hash_formatted = format!("sha256:{file_hash:x}");

        DataPackageResource {
            name: file_name,
            path,
            hash: file_hash_formatted,
            bytes: file_bytes.len(),
        }
    }
    pub fn create_digest(data_package_file: &[u8]) -> DataPackageDigest {
        let file_hash = Sha256::digest(data_package_file);
        let file_hash_formatted = format!("sha256:{file_hash:x}");
        DataPackageDigest {
            path: "datapackage.json".to_owned(),
            hash: file_hash_formatted,
        }
    }
}

pub fn create_datapackage(warc_file: &[u8]) -> Vec<u8> {
    let mut resources = Vec::with_capacity(1);

    // this could be a loop, with more things
    let path: &Path = Path::new("archive/data.warc");
    let resource = DataPackage::create_resource(path, warc_file.to_vec());
    resources.push(resource);

    let data_package = DataPackage::new(resources);

    // Serialize it to JSON byte array
    serde_json::to_vec(&data_package).unwrap()
}
