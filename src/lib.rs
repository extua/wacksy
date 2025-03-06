pub mod zip_dir;
pub use zip_dir::*;
pub mod datapackage;
pub use datapackage::*;

const WACZ_VERSION: &str = "1.1.1";

pub struct Wacz {
    pub warc_file: Vec<u8>,
    pub data_package: Vec<u8>,
    pub data_package_digest_bytes: Vec<u8>,
}
