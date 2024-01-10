# `alpha-g-vertices`

The `alpha-g-vertices` program will produce a CSV file with all the 
reconstructed vertices of a run.

The CSV data will have the following three-line header:

```
# Version of the alpha-g-analysis package
# CLI arguments used to run the program
serial_number,trg_time,reconstructed_x,reconstructed_y,reconstructed_z
```

The trigger time is in seconds and the reconstructed coordinates of the
vertex are in meters. Note that an event might not have a reconstructed
vertex, in which case the coordinates fields will be empty. Furthermore, if
there is an error with a particular event, the `trg_time` field might be
empty as well. The serial number is a unique identifier for the event and it
will always be present in the CSV output even if the event cannot be processed.
