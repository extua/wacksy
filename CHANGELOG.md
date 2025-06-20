# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1-beta.1](https://github.com/extua/wacksy/compare/v0.0.1-beta...v0.0.1-beta.1) - 2025-06-20

### Added

- add pages indexer to wacz writer, closes #30
- *(indexer)* add struct for page records

### Fixed

- add newline to page records, needed for pages.jsonl format, closes #37
- *(indexer)* skip serialising null fields in page record
- *(datapackage)* add cdxj_index_bytes through to the datapackage
- pages.jsonl path is under pages directory
- *(citation)* citation name the wrong way around

### Other

- explain datapackage digest struct
- hide unimplemented PageTitle type from documentation
- *(indexer)* use core instead of standard libraries for error formatting
- *(indexer)* add inline documentation for all indexer functions, closes #34
- *(indexer)* rename CDXJIndexError to more generic IndexingError
- *(indexer)* revert borrowing where unnecessary
- initial work to index pages
- add serde features to dependencies, update cargofile
- *(deps)* bump rawzip from 0.1.0 to 0.2.0 in the minor-bumps group ([#35](https://github.com/extua/wacksy/pull/35))
- *(indexer)* more documentation, changed error types for CDXJIndex
- *(indexer)* use core::error rather than std::error
- link bodleian site in readme
- don't re-export datapackage types
- move zip_dir into Wacz object trait
- *(indexer)* clarify shadowed variables
- *(datapackage)* move compose_datapackage into datapackage implementation
- *(indexer)* add documentation to functions
- *(datapackage)* DataPackageResource::new now returns a result/error rather than panicking
- *(citation)* add reference to WACZ standard, fixes #7
- add documentation to types
- *(indexer)* use type generics to eliminate code duplication
- update cargofile with release profile flags
- *(indexer)* use httparse to parse http status code from response
- *(indexer)* remove the happily redundant cut_http_headers_from_record function
- add info and badges to readme

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
