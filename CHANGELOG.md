# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1](https://github.com/extua/wacksy/compare/v0.0.1-beta...v0.0.1) - 2025-06-03

### Added

- Add preliminary pages indexer to WACZ writer and refactored the indexer so it produces pages.jsonl and index.cdxj in the same pass through the file.
- *(indexer)* Use type generics to eliminate code duplication when iterating through records, this finally gets rid of an awkward situation where I was having to maintain two separate iterators .

### Other

- *(docs)* The other main thing in this release is that all functions in the indexer have inline documentation.
- *(indexer)* use `core::error` rather than `std::error`
- Move zip_dir into Wacz object trait.
- *(datapackage)* DataPackageResource::new now returns a result/error rather than panicking.
- *(indexer)* Use httparse to parse http status code from response and remove the happily redundant cut_http_headers_from_record function.

There are still opportunities to panic in `lib.rs`.

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
