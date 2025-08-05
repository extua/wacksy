# Wacksy

[![SWH](https://archive.softwareheritage.org/badge/swh:1:snp:4946a5fe3e15fa0c3e9e68cd5b3816cf80f733d6/)](https://archive.softwareheritage.org/swh:1:snp:4946a5fe3e15fa0c3e9e68cd5b3816cf80f733d6;origin=https://github.com/extua/wacksy)

An experimental Rust library for ~~reading and~~ writing WACZ files.

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

### Similar libraries

* [py-wacz](https://github.com/webrecorder/py-wacz) for python
* [js-wacz](https://github.com/harvard-lil/js-wacz) for javascript

## License

[MIT](https://github.com/extua/wacksy/blob/main/LICENSE) © [Bodleian Libraries](https://www.bodleian.ox.ac.uk/) and contributors
