use std::{fs, path::Path};
use wacksy::{DataPackage, Wacz, create_datapackage, datapackage};

fn main() {
    // let data_package: Wacz = {
    //     let warc_file = fs::read("examples/warc_example.warc").unwrap();
    //     Wacz { warc_file }
    // };
    // // This needs to be parsed into a file!
    // let wacz_data = zip_dir(&wacz_object).unwrap();
    // let path: &Path = Path::new("datapackage.json");

    // fs::write(path, wacz_data).unwrap();
    let path: &Path = Path::new("datapackage.json");
    let data_package = create_datapackage();
    fs::write(path, data_package).unwrap();
}
