use std::path::Path;
use std::{error::Error, fs};
use wacksy::WACZ;

fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    let warc_file_path: &Path = Path::new("examples/rec-e7e68da067d0-20250423121042981-0.warc.gz");

    let wacz_object = WACZ::from_file(warc_file_path).unwrap();
    let zipped_wacz = wacz_object.zip().unwrap();

    let path: &Path = Path::new("wacz_example.wacz");
    fs::write(path, zipped_wacz)?;
    Ok(())
}
