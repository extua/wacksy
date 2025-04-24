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

## Operations chart

Broadly what needs to be done, read the WACZ file, create an index and, a datapackage, _in that order_ and then convert everything to bytes and zip it up.

```mermaid
flowchart
    A@{ shape: lean-r, label: "WARC file"}
    B@{ shape: rect, label: "Create index" }
    C@{ shape: rect, label: "Create datapackage" }
    D@{ shape: rect, label: "Create datapackage digest" }
    E1@{ shape: lean-l, label: "Convert index to bytes" }
    E2@{ shape: lean-l, label: "Convert to bytes" }
    F@{ shape: lean-l, label: "Zip up the files" }
    G@{ shape: lean-r, label: "WACZ file"}
    A --> index
    subgraph index
    B --> E1
    end
    index --> datapackage
    subgraph datapackage
    C --> E2 --> D --> E2
    style index stroke-dasharray: 5 5
    end
    A --> F
    index --> F
    datapackage --> F --> G
```
