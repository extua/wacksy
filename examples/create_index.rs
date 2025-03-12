use std::path::Path;

use wacksy::compose_index;

fn main() {
    let warc_file_path: &Path =
        Path::new("../crawl_example/archive/rec-68fed303f9b7-20250312124948172-0.warc");

    let index = match compose_index(warc_file_path) {
        Ok(index) => index,
        Err(error) => panic!("Problem opening the file: {error:?}"),
    };
    index
}
