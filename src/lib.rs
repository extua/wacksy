#[forbid(unsafe_code)]
pub mod zip_dir;
pub use zip_dir::*;
pub mod datapackage;
pub use datapackage::*;
pub mod indexer;

const WACZ_VERSION: &str = "1.1.1"; // deprecated in WACZ 1.2.0

pub struct Wacz {
    pub warc_file: Vec<u8>,
    pub data_package_bytes: Vec<u8>,
    pub data_package_digest_bytes: Vec<u8>,
    pub index_bytes: Vec<u8>,
}
