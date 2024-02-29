# `alpha-g-chronobox-timestamps`

The `alpha-g-chronobox-timestamps` program will produce a CSV file with all the 
Chronobox timestamps of a run.

The CSV data will have the following three-line header:

```
# <package_name> <package_version>
# <cli_arguments>
board,channel,leading_edge,chronobox_time
```

The Chronobox time is in seconds. The `chronobox_time` field might be empty if
there is an issue with a particular timestamp. All the other fields will always
be present in the CSV output even if the timestamp cannot be processed. Note
that the data is NOT sorted by the `chronobox_time` field, data is grouped by
board name and (within each board) written in the order it appears in the MIDAS
input files.
