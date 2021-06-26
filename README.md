# F-Chat 3.0 Log Merger

With two or more log locations as inputs, it will merge files by comparing messages and see if any of them should be saved or not if they are the same content **and** within a certain amount of time.

```
fchat3-log-merger 1.0
Carlen White <whitersuburban@gmail.com>
Reads multiple F-Chat 3.0 client log folders and merges them together

USAGE:
    fchat3-log-merger [OPTIONS] --folder <folders>...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --folder <folders>...      What folders to read from.
    -o, --output <output>          Folder to write the merged logs to.
    -d, --time-diff <time-diff>    How long the time difference between messages to check for duplicates specified in
                                   human time. Defaults to 5min
```