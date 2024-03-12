# `alpha-g-sequencer`

The `alpha-g-sequencer` program will produce a CSV file with all the 
sequencer data of a run.

The CSV data will have the following three-line header:

```
# <package_name> <package_version>
# <cli_arguments>
serial_number,midas_timestamp,header,xml
```

Note that the `header` and `xml` fields are arbitrary UTF-8 strings i.e. they
can contain line breaks, commas, quotes, etc. These fields are enclosed in
double quotes and any double quotes within the field are escaped with another
double quote.
