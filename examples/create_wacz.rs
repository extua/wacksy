use std::{fs, path::Path};
use wacksy::{Wacz, zip_dir};

fn main() {
    let wacz_object: Wacz = {
        let warc_file = fs::read("examples/warc_example.warc").unwrap();
        Wacz { warc_file }
    };
    // This needs to be parsed into a file!
    let wacz_data = zip_dir(&wacz_object).unwrap();
    let path: &Path = Path::new("wacz_example.wacz");

    fs::write(path, wacz_data).unwrap();
}
