use std::{fs::File, io::Write};

fn main() {
    let buffer = File::create("foo.zip").expect("couldn't create file");
    let mut writer = rawzip::ZipArchiveWriter::new(buffer);
    writer.new_dir("dir/").unwrap();

    let options =
        rawzip::ZipEntryOptions::default().compression_method(rawzip::CompressionMethod::Store);

    let mut file = writer.new_file("dir/test.txt", options).unwrap();
    let output = {
        let mut writer = rawzip::RawZipWriter::new(&mut file);
        writer.write_all(b"Hello, world!").unwrap();
        let (_, output) = writer.finish().unwrap();
        output
    };
    file.finish(output).unwrap();
    writer.finish().unwrap();
}