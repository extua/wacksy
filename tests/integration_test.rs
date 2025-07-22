use std::{fs, path::Path};

const WARC_PATH: &str = "tests/rec-e7e68da067d0-20250423121042981-0.warc.gz";

#[test]
fn test_cdxj_indexer() -> Result<(), std::io::Error> {
    let warc_file_path: &Path = Path::new(WARC_PATH);
    let index = wacksy::indexer::index_file(warc_file_path)?;
    let generated_cdxj_index = index.0.to_string();
    let example_cdxj_index = fs::read_to_string(Path::new("tests/index.cdxj"))?;
    assert_eq!(generated_cdxj_index, example_cdxj_index);
    Ok(())
}
