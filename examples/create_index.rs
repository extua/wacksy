use std::path::Path;

use wacksy::compose_index;

fn main() {
    let warc_file_path: &Path = Path::new("examples/warc_example.warc");

    let index = match compose_index(warc_file_path) {
        Ok(index) => index,
        Err(error) => panic!("Problem opening the file: {error:?}"),
    };
    index
}
