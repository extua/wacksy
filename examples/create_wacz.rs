use std::{
    fs,
    path::{self, Path},
};
use wacksy::{Wacz, create_datapackage, zip_dir};

fn main() {
    let warc_file = fs::read("examples/warc_example.warc").unwrap();
    let data_package = create_datapackage(&warc_file);

    let wacz_object: Wacz = {
        Wacz {
            warc_file,
            data_package,
        }
    };

    // This needs to be parsed into a file!
    let wacz_data: Vec<u8> = zip_dir(&wacz_object).unwrap();
    let path: &Path = Path::new("wacz_example.wacz");
    fs::write(path, wacz_data).unwrap();
}
