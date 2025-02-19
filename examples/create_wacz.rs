use std::fs::File;
use wacksy::zip_dir;

fn main() {
    let wacz_file = File::create("foo.zip").unwrap();
    zip_dir(wacz_file);
}
