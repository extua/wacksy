# Wacksy

[![SWH](https://archive.softwareheritage.org/badge/swh:1:dir:ce81998f0400b211fff19416668bb22d9c53c441/)](https://archive.softwareheritage.org/swh:1:dir:ce81998f0400b211fff19416668bb22d9c53c441;origin=https://github.com/extua/wacksy;visit=swh:1:snp:701dca05ae362b4f3de5d31a2ad5387fa68f02cf;anchor=swh:1:rev:1d054902a74084c8e67798a561d7efe573418e70)

An experimental Rust library for reading and writing WACZ files.

<!-- purpose of this library -->

## Install

With cargo installed, run the following command in your project directory:

```
cargo add wacksy
```

## Background

A WACZ file is essentially [a zip file](https://bikeshed.vibber.net/@brooke/114240574949828718); according to [the example in the spec](https://specs.webrecorder.net/wacz/1.1.1/) it should contain:

```
archive
└── data.warc.gz
datapackage.json
datapackage-digest.json
indexes
└── index.cdx.gz
pages
└── pages.jsonl
```

## License

[MIT](https://github.com/extua/wacksy/blob/main/LICENSE) © Bodleian Libraries and contributors
