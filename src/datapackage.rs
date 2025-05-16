use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::WACZ_VERSION;

/// This struct defines a [frictionless
/// datapackage](https://specs.frictionlessdata.io/data-package/).
#[derive(Serialize, Deserialize)]
pub struct DataPackage {
    pub profile: String,
    pub wacz_version: String,
    pub created: String,
    pub software: String,
    pub resources: Vec<DataPackageResource>,
}

/// A datapackage resource is anything which needs
/// to be defined in the datapackage
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

impl Default for DataPackage {
    fn default() -> Self {
        return Self {
            profile: "data-package".to_owned(),
            wacz_version: WACZ_VERSION.to_owned(),
            created: Local::now().to_rfc3339(),
            software: format!("wacksy {}", env!("CARGO_PKG_VERSION")),
            resources: Vec::with_capacity(512),
        };
    }
}
impl DataPackage {
    fn new() -> Self {
        return Self::default();
    }
    pub fn add_resource(data_package: &mut Self, resource: DataPackageResource) {
        data_package.resources.push(resource);
    }

    /// # Digest datapackage
    ///
    /// Takes a `DataPackage` struct and returns a `DataPackageDigest`
    /// containing an sha256 hash of the datapackage.
    ///
    /// # Errors
    ///
    /// Will return a `serde_json` error if there's any problem
    /// deserialising the data package to a vector.
    pub fn digest(data_package: &Self) -> Result<DataPackageDigest, serde_json::Error> {
        let data_package_file = serde_json::to_vec(&data_package)?;
        let file_hash = Sha256::digest(data_package_file);
        let file_hash_formatted = format!("sha256:{file_hash:x}");
        return Ok(DataPackageDigest {
            path: "datapackage.json".to_owned(),
            hash: file_hash_formatted,
        });
    }
}

// A singular resource
impl DataPackageResource {
    #[must_use]
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

        return Self {
            name: file_name,
            path,
            hash: file_hash_formatted,
            bytes: file_bytes.len(),
        };
    }
}

/// The datapackage should look something like this when written out to file:
/// 
/// ```json
/// {
///   "profile": "data-package",
///   "wacz_version": "1.1.1",
///   "created": "2025-05-16T11:03:03.499792020+01:00",
///   "software": "wacksy 0.0.1-alpha",
///   "resources": [
///     {
///       "name": "data.warc",
///       "path": "archive/data.warc",
///       "hash": "sha256:210d0810aaf4a4aba556f97bc7fc497d176a8c171d8edab3390e213a41bed145",
///       "bytes": 4599
///     },
///     {
///       "name": "index.cdxj",
///       "path": "indexes/index.cdxj",
///       "hash": "sha256:0494f16f39fbb3744556e1d64be1088109ac35c730f4a30ac3a3b10942340ca3",
///       "bytes": 543
///     }
///   ]
/// }
/// ```
/// 
/// [Link to spec](https://specs.webrecorder.net/wacz/1.1.1/#datapackage-json)
#[must_use]
pub fn compose_datapackage(warc_file: &[u8], index_file: &[u8]) -> DataPackage {
    let mut data_package = DataPackage::new();

    // this _could_ be a loop, with more things
    // add warc file to datapackage
    let path: &Path = Path::new("archive/data.warc");
    let resource = DataPackageResource::new(path, warc_file);
    DataPackage::add_resource(&mut data_package, resource);

    // add index file to datapackage
    let path: &Path = Path::new("indexes/index.cdxj");
    let resource = DataPackageResource::new(path, index_file);
    DataPackage::add_resource(&mut data_package, resource);

    return data_package;
}
