use std::{fs, path::Path};
use wacksy::zip_dir;

fn main() {
    let warc_file = fs::read("examples/warc_example.warc").unwrap();
    let wacz_data = zip_dir(&warc_file).unwrap();
    let path: &Path = Path::new("wacz_example.wacz");
    fs::write(path, wacz_data).unwrap();
}
