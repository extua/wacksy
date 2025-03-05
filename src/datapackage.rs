use std::path::{self, Path};

use serde::{Deserialize, Serialize};
use serde_json;

use crate::WACZ_VERSION;

// Link to the spec
// https://specs.webrecorder.net/wacz/1.1.1/#datapackage-json

#[derive(Serialize, Deserialize)]
pub struct DataPackage {
    pub profile: String,
    #[serde(rename = "wacz_version")]
    pub wacz_version: String,
    pub resources: Vec<DataPackageResource>,
}

#[derive(Serialize, Deserialize)]
pub struct DataPackageResource {
    pub name: String,
    // pub path: String,
    // pub hash: String,
    // pub bytes: usize,
}

impl DataPackage {
    pub fn new(resources: Vec<DataPackageResource>) -> Self {
        let data_package = DataPackage {
            profile: "data-package".to_owned(),
            wacz_version: WACZ_VERSION.to_owned(),
            resources: resources,
        };
        data_package
    }
}

impl DataPackageResource {
    pub fn new(path: &Path, file_bytes: Vec<u8>) -> Self {
        // to build this we give it a path and some bytes to read?

        let resource = DataPackageResource {
            // something really needs to be done about this
            // horrible chain of things going on here
            name: path.file_name().unwrap().to_str().unwrap().to_owned(),
            // path: todo!(),
            // hash: todo!(),
            // bytes: todo!(),
        };
        resource
    }
}

pub fn create_datapackage(warc_file: &Vec<u8>) -> Vec<u8> {
    let path: &Path = Path::new("archive/data.warc");
    let resource = DataPackageResource::new(path, warc_file.to_vec());

    let mut resources = Vec::with_capacity(1);
    resources.push(resource);

    let data_package = DataPackage::new(resources);

    // Serialize it to JSON byte array
    let data_package_bytes: Vec<u8> = serde_json::to_vec(&data_package).unwrap();

    data_package_bytes
}
