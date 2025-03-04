use serde::{Deserialize, Serialize};
use serde_json;

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
    pub path: String,
    pub hash: String,
    pub bytes: i64,
}

impl DataPackage {
    pub fn new() -> Self {
        let data_package = DataPackage {
            profile: "data-package".to_owned(),
            wacz_version: "1.1.1".to_owned(),
            resources: Vec::new(),
        };
        data_package
    }
}

pub fn create_datapackage() -> Vec<u8> {
    let data_package = DataPackage::new();

    // Serialize it to JSON byte array
    let data_package_bytes: Vec<u8> = serde_json::to_vec(&data_package).unwrap();

    data_package_bytes
}
