# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.1-alpha.1](https://github.com/extua/wacksy/compare/v0.0.1-alpha...v0.0.1-alpha.1) - 2025-04-17

### Other

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
