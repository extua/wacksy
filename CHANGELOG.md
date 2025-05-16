# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
