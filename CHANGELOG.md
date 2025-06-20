# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1](https://github.com/extua/wacksy/compare/v0.0.1-beta...v0.0.1) - 2025-06-20

As of this point, the WACZ and indexer can output (almost) everything needed from a WARC file to a fully spec-compliant WACZ file.
The last thing missing was the pages.jsonl file, which is now produced when reading through the WARC file as part of the indexer.
I want to avoid reading through the WARC twice to produce two files, so have wrapped everything into one indexer, again there's probably a better way of doing this.

The other happy change in this release is removing code duplication from the WARC reader in case of gzipped and non-gzipped files.
I learned about generics!

### Added

- add pages indexer to wacz writer, with a struct for page records, this is the main thing in this release.

### Fixed

- add newline to page records, needed for pages.jsonl format, closes #37, nice and easy change
- *(indexer)* skip serialising null fields in page record
- *(datapackage)* pass cdxj_index_bytes through to the datapackage

### Other

Lots more little documentation/readme changes and additions. Code refactoring, etc.

- *(indexer)* use core instead of standard libraries for error formatting
- add serde features to dependencies, update cargofile
- *(datapackage)* move compose_datapackage into datapackage implementation
- *(datapackage)* DataPackageResource::new now returns a result/error rather than panicking
- *(indexer)* use httparse to parse http status code from response and remove the happily redundant cut_http_headers_from_record function

## [0.0.1-beta](https://github.com/extua/wacksy/compare/v0.0.1-alpha...v0.0.1-beta) - 2025-05-16

Work on this version was mostly refactoring, adding structured types and error handling, and some documentation (only just started).

Still on my todo list is to use the indexer to also create pages.jsonl files.

### Fixed

- replace wrapping_add in loop counter with enumerate, closes #29
- *(indexer)* return the same error message for gzipped and non-gzipped files. I have tried to simplify the code for processing both gzipped and non-gzipped files. There's still unnecessary duplication but it's the best I can do for the moment.

### Other

- document some DataPackage structs, better documentation coming once this is properly finished!
- as a style change, this now uses explicit returns everywhere, and I have set lints in cargo.toml to enforce this
- *(indexer)* many of the index functons are now implemented on types. The completed index is returned *as a struct*, which has a display implementation to write it out to json(l).
- *(datapackage)* propogate errors upwards, there are still some panics, but structured error handling is a lot more comprehensive now. Happy and unhappy paths are a little clearer to identify.
- update README with link to a funny meme :)

## [0.0.1-alpha](https://github.com/extua/wacksy/releases/tag/v0.0.1-alpha) - 2025-04-05

At this stage the library can read a WARC file to produce a CDXJ index, and a datapackage.

### Added

- *(indexer)* types for DataPackage and DataPackageResource
- *(indexer)* various types for CXDJIndexRecord
