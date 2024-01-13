# `alpha-g-trg-scalers`

The `alpha-g-trg-scalers` program will produce a CSV file with all the 
TRG scalers of a run.

The CSV data will have the following three-line header:

```
# <package_name> <package_version>
# <cli_arguments>
serial_number,trg_time,input,drift_veto,scaledown,pulser,output
```

The trigger time is in seconds and all the scaler counts are unsigned 32-bit
integers. If there is an error with a particular event, the `trg_time` field
might be empty. Furthermore, different versions of the TRG data banks might
not have all the scalers. In that case, the header will still have all the
fields but the corresponding columns will be empty. The serial number is a
unique identifier for the event and it will always be present in the CSV output
even if the event cannot be processed.
