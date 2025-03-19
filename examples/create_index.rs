use std::path::Path;

use wacksy::compose_index;

fn main() {
    let warc_file_path: &Path =
        Path::new("../warc_examples/rec-20220831121513589208-203de340fdad.warc.gz");

    let index = match compose_index(warc_file_path) {
        Ok(index) => index,
        Err(error) => panic!("Problem opening the file: {error:?}"),
    };
    index
}
