# F-Chat 3.0 Log Merger

With two or more log locations as inputs, it will merge files by comparing messages and see if any of them should be saved or not if they are the same content **and** within a certain amount of time.

```
fchat3-log-merger 1.2.0 
Carlen White <whitersuburban@gmail.com>
 
Reads multiple F-Chat 3.0 client log folders and merges them together
 
 Usage: fchat3-log-merger [OPTIONS] --folders <FOLDERS> <FOLDERS>... 
 Options:
  -f, --folders <FOLDERS> <FOLDERS>...
          What folders to read from
  -d, --time-diff <TIME_DIFF>
          How long the time difference between messages to check for duplicates
          specified in human time [default: 0s]
      --fast-forward <FAST_FORWARD>
          Assuming the left-most is up-to-date, skip to this timestamp in
          YYYY-MM-DD HH:MM:SS
  -o, --output <OUTPUT>
          Folder to write the merged logs to
      --dry-run
          Collects files, but does not do anything
      --dupe-warning
          Indicate if a file has more than one duplicate messages in the
          comparison window
  -v...
          Increase verbosity. More occurances increases the verbosity
  -h, --help
          Print help
  -V, --version
          Print version
```