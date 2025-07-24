//! Types for defining a datapackage.json file.
//!
//! The file should look something like this when complete:
//!
//! ```json
//! {
//!   "profile": "data-package",
//!   "wacz_version": "1.1.1",
//!   "created": "2025-07-23T19:11:12.817986724+01:00",
//!   "software": "wacksy 0.0.1",
//!   "resources": [
//!     {
//!       "name": "data.warc",
//!       "path": "archive/data.warc",
//!       "hash": "blake3:b33e5bd9e2cf814a74ffa51256591f58fb0d95e5068d2121cf2775179d0ffd90",
//!       "bytes": 4599
//!     },
//!     {
//!       "name": "index.cdxj",
//!       "path": "indexes/index.cdxj",
//!       "hash": "blake3:d388a3aee3afd383d50bd1167f6f33b60017bc37361c93733dc25fcc83aeac6e",
//!       "bytes": 543
//!     }
//!   ]
//! }
//! ```
//!
//! [Link to spec](https://specs.webrecorder.net/wacz/1.1.1/#datapackage-json)

use blake3::hash;
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{error::Error, path::Path};

use crate::WACZ_VERSION;

/// A [frictionless datapackage](https://specs.frictionlessdata.io/data-package/).
#[derive(Serialize, Deserialize)]
pub struct DataPackage {
    pub profile: String,
    pub wacz_version: String,
    pub created: String,
    pub software: String,
    pub resources: Vec<DataPackageResource>,
}

/// A datapackage resource is anything which needs
/// to be defined in the datapackage.
#[derive(Serialize, Deserialize)]
pub struct DataPackageResource {
    pub name: String,
    pub path: String,
    pub hash: String,
    pub bytes: usize,
}

/// A digest of the datapackage file itself.
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
    /// # Create datapackage
    ///
    /// Composes the data package and adds resources to it.
    ///
    /// # Errors
    ///
    /// Will return a `DataPackageError` relating to any
    /// resource if there is anything wrong with the filename
    /// or path of a resource.
    pub fn new(
        warc_file: &[u8],
        cdxj_index_file: &[u8],
        pages_index_file: &[u8],
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let mut data_package = Self::default();

        // this _could_ be a loop, with more things
        // add warc file to datapackage
        let path: &Path = Path::new("archive/data.warc");
        let resource = DataPackageResource::new(path, warc_file)?;
        Self::add_resource(&mut data_package, resource);

        // add index file to datapackage
        let path: &Path = Path::new("indexes/index.cdxj");
        let resource = DataPackageResource::new(path, cdxj_index_file)?;
        Self::add_resource(&mut data_package, resource);

        // add index file to datapackage
        let path: &Path = Path::new("pages/pages.jsonl");
        let resource = DataPackageResource::new(path, pages_index_file)?;
        Self::add_resource(&mut data_package, resource);

        return Ok(data_package);
    }

    /// Takes a `DataPackage` struct and pushes a resource to the
    /// 'resources' field.
    fn add_resource(data_package: &mut Self, resource: DataPackageResource) {
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
        let file_hash = hash(&data_package_file).to_hex();
        let file_hash_formatted = format!("blake3:{file_hash}");

        return Ok(DataPackageDigest {
            path: "datapackage.json".to_owned(),
            hash: file_hash_formatted,
        });
    }
}

impl DataPackageResource {
    /// # Instantiate datapackage resource
    ///
    /// This is for serialising a single resource to
    /// a struct to pass through to the `DataPackage`.
    ///
    /// # Errors
    ///
    /// Will return a `DataPackageError` mainly in case the
    /// resource file path or file name are missing or cannot
    /// be converted to string.
    pub fn new(path: &Path, file_bytes: &[u8]) -> Result<Self, DataPackageError> {
        let file_name = match path.file_name() {
            Some(file_name) => match file_name.to_str() {
                Some(file_name) => file_name.to_owned(),
                None => {
                    return Err(DataPackageError::FileNameError(format!(
                        "unable to convert {file_name:?} to string"
                    )));
                }
            },
            None => {
                return Err(DataPackageError::FileNameError(
                    "file name is empty".to_owned(),
                ));
            }
        };
        let path = match path.to_str() {
            Some(path) => path.to_owned(),
            None => {
                return Err(DataPackageError::FilePathError(format!(
                    "unable to convert {file_name:?} to string"
                )));
            }
        };

        // create file hash for the resource
        let file_hash = hash(file_bytes).to_hex();
        let file_hash_formatted = format!("blake3:{file_hash}");

        return Ok(Self {
            name: file_name,
            path,
            hash: file_hash_formatted,
            bytes: file_bytes.len(),
        });
    }
}

#[derive(Debug)]
pub enum DataPackageError {
    FileNameError(String),
    FilePathError(String),
}
impl std::fmt::Display for DataPackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNameError(error_message) => {
                return write!(f, "Filename error: {error_message}");
            }
            Self::FilePathError(error_message) => {
                return write!(f, "File path error: {error_message}");
            }
        }
    }
}
impl Error for DataPackageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FilePathError(_) | Self::FileNameError(_) => return None,
        }
    }
}
