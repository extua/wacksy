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

// Higher level data package

impl Default for DataPackage {
    fn default() -> Self {
        DataPackage {
            profile: "data-package".to_owned(),
            wacz_version: WACZ_VERSION.to_owned(),
            // There will be at least two resources in
            // any WACZ file, the jsonl file and
            // the WARC file
            resources: Vec::with_capacity(2),
        }
    }
}
impl DataPackage {
    fn new() -> Self {
        DataPackage::default()
    }
    pub fn add_resource(data_package: &mut DataPackage, resource: DataPackageResource) {
        data_package.resources.push(resource);
    }
    pub fn digest(data_package: &Self) -> DataPackageDigest {
        let data_package_file = serde_json::to_vec(&data_package).unwrap();
        let file_hash = Sha256::digest(data_package_file);
        let file_hash_formatted = format!("sha256:{file_hash:x}");
        DataPackageDigest {
            path: "datapackage.json".to_owned(),
            hash: file_hash_formatted,
        }
    }
}

// A singular resource
impl DataPackageResource {
    pub fn new(path: &Path, file_bytes: &[u8]) -> Self {
        // handle the option-result, but there's not
        // much to be done about this unfortunately
        let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let path = path.to_str().unwrap().to_owned();

        // create a sha256 hash, from documentation
        // here https://docs.rs/sha2/latest/sha2/
        // create a Sha256 object
        let file_hash = Sha256::digest(file_bytes);
        let file_hash_formatted = format!("sha256:{file_hash:x}");

        DataPackageResource {
            name: file_name,
            path,
            hash: file_hash_formatted,
            bytes: file_bytes.len(),
        }
    }
}

pub fn compose_datapackage(warc_file: &[u8]) -> DataPackage {
    let mut data_package = DataPackage::new();

    // this _could_ be a loop, with more things
    let path: &Path = Path::new("archive/data.warc");
    let resource = DataPackageResource::new(path, warc_file);
    DataPackage::add_resource(&mut data_package, resource);

    data_package
}
