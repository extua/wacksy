//! Types for defining a datapackage.json file.
//!
//! The file should look something like this when complete:
//!
//! ```json
//! {
//!   "profile": "data-package",
//!   "wacz_version": "1.1.1",
//!   "created": "2025-05-16T11:03:03.499792020+01:00",
//!   "software": "wacksy 0.0.1-beta",
//!   "resources": [
//!     {
//!       "name": "data.warc",
//!       "path": "archive/data.warc",
//!       "hash": "sha256:210d0810aaf4a4aba556f97bc7fc497d176a8c171d8edab3390e213a41bed145",
//!       "bytes": 4599
//!     },
//!     {
//!       "name": "index.cdxj",
//!       "path": "indexes/index.cdxj",
//!       "hash": "sha256:0494f16f39fbb3744556e1d64be1088109ac35c730f4a30ac3a3b10942340ca3",
//!       "bytes": 543
//!     }
//!   ]
//! }
//! ```
//!
//! [Link to spec](https://specs.webrecorder.net/wacz/1.1.1/#datapackage-json)

use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest as _, Sha256};
use std::{error::Error, ffi::OsStr, fmt, fs, path::Path};

use crate::{WACZ_VERSION, indexer::Index};

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
#[derive(Serialize, Deserialize, Debug)]
pub struct DataPackageResource {
    #[serde(rename = "name")]
    pub file_name: String,
    pub path: String,
    pub hash: String,
    pub bytes: usize,
    #[serde(skip)]
    pub content: Vec<u8>,
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
        warc_file_path: &Path,
        index: &Index,
    ) -> Result<Self, Box<dyn Error + Send + Sync + 'static>> {
        let mut data_package = Self::default();

        let warc_file_bytes = fs::read(warc_file_path).unwrap();

        // add warc file to datapackage
        let path: &Path = if warc_file_path.extension() == Some(OsStr::new("gz")) {
            Path::new("archive/data.warc.gz")
        } else {
            Path::new("archive/data.warc")
        };
        let resource = DataPackageResource::new(path, &warc_file_bytes)?;
        Self::add_resource(&mut data_package, resource);

        // add cdxj file to datapackage
        let path: &Path = Path::new("indexes/index.cdxj");
        let resource = DataPackageResource::new(path, &index.cdxj.to_string().into_bytes())?;
        Self::add_resource(&mut data_package, resource);

        // add pages file to datapackage
        let path: &Path = Path::new("pages/pages.jsonl");
        let resource = DataPackageResource::new(path, &index.pages.to_string().into_bytes())?;
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
    pub fn digest(&self) -> Result<DataPackageDigest, serde_json::Error> {
        return Ok(DataPackageDigest {
            path: "datapackage.json".to_owned(),
            hash: format!("sha256:{:x}", Sha256::digest(serde_json::to_vec(&self)?)),
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

        return Ok(Self {
            file_name,
            path,
            hash: format!("sha256:{:x}", Sha256::digest(file_bytes)),
            bytes: file_bytes.len(),
            content: file_bytes.to_vec(),
        });
    }
}

#[derive(Debug)]
pub enum DataPackageError {
    FileNameError(String),
    FilePathError(String),
}
impl fmt::Display for DataPackageError {
    fn fmt(&self, message: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNameError(error_message) => {
                return write!(message, "Filename error: {error_message}");
            }
            Self::FilePathError(error_message) => {
                return write!(message, "File path error: {error_message}");
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
