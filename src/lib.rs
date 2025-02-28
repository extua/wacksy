pub mod zip_dir;
pub use zip_dir::*;

pub struct Wacz {
    pub warc_file: Vec<u8>,
}
