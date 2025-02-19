use std::{fs, fs::File};
use wacksy::zip_dir;

fn main() {
    let wacz_file = File::create("foo.zip").unwrap();
    let warc_file = fs::read("examples/warc_example.warc").unwrap();
    zip_dir(wacz_file, &warc_file);
}
