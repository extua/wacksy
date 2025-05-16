# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1-alpha.1](https://github.com/extua/wacksy/compare/v0.0.1-alpha...v0.0.1-alpha.1) - 2025-05-16

### Fixed

- replace wrapping_add in loop counter with enumerate, closes #29
- *(indexer)* return the same error message for gzipped and non-gzipped files

### Other

- bump citation version ready for release
- remove operations chart in readme
- document some DataPackage structs
- add semicolons to explicit return statements
- add documentation to zipper
- make everything return explicitly
- fix various lints
- add lints to cargofile
- downgrade rust version to use arm runner
- use arm runners, cache pip
- bump deps
- *(indexer)* move compose_index into CDXJIndex type
- *(indexer)* return a list of records rather than some bytes
- *(indexer)* move gzip and non-gzip loops back into compose_index
- match indexer record loop to  clear happy and unhappy paths
- remove unnecessary structure name repetition for DataPackage
- replace CDXJIndexRecordError with Self
- *(deps)* bump the minor-bumps group across 1 directory with 2 updates ([#28](https://github.com/extua/wacksy/pull/28))
- *(datapackage)* propogate errors upwards
- *(indexer)* println and break on error rather than panic if a record is bad
- update README with link to a funny meme
- change example used for creating files
- *(zipper)* we are creating cdxj indexes, not cdx
- *(indexer)* move indexing errors into their own module
- *(indexer)* return httparse parsing error to string in RecordContentType::new()
- *(indexer)* bubble up errors from compose_index
- *(indexer)* finish implementing most of the indexer error types
- *(indexer)* continue to specify source when implementing Error
- bump MSRV to 1.85.1
- *(indexer)* add more error types for the indexer
- *(indexer)* remove type definitions from indexer.rs
- *(indexer)* move WARC record types into their own module
- *(lib)* pro-actively forbid unsafe code
- *(indexer)* use writeln instead of adding a newline to the string

## [0.0.1-alpha](https://github.com/extua/wacksy/releases/tag/v0.0.1-alpha) - 2025-04-05

At this stage the library can read a WARC file to produce a CDXJ index, and a datapackage.

### Added

- *(indexer)* types for DataPackage and DataPackageResource
- *(indexer)* various types for CXDJIndexRecord
