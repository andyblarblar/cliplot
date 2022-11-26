# cliplot 

A command line application for live-plotting data.

![](docs/demo.gif)

Features:
- Live plotting from stdin
- 1-many input channels
- Configurable regex for each channel
- Log data to CSV while plotting
- Cross-platform

## Usage

```
cliplot will live plot data piped into stdin. It can plot multiple channels, parse data with regex, save data to a CSV, and more.

Usage: cliplot [OPTIONS]

Options:
  -r, --regexes <REGEXES>
          Regex strings to parse each channel with. If this is not specified, then a single channel that parses for `$float$` will be used.
          
          Each regex should be unambiguous from the others, and contain one capture group that contains a string convertable to a f64. Deliminators (such as the `$` above) are necessary to avoid numbers being cut across buffer breaks.

      --csv <CSV>
          Writes read data into a CSV file at path if set.
          
          The CSV file will contain the timestamp of each reading in ms, followed by the data and finally the channel number.

  -v, --verbose...
          More output per occurrence

  -q, --quiet...
          Less output per occurrence

  -h, --help
          Print help information (use `-h` for a summary)

  -V, --version
          Print version information

```

- cliplot uses [Rust regex syntax](https://docs.rs/regex/latest/regex/#syntax). Be careful with shell interpretations of symbols like $.
- Some shells like bash have issues with multiple regex strings. In this case, just use multiple `-r` flags in order.
- To double-check your regex is being interpreted correctly, run with -vvv. To see the data being parsed, run with -vvvv.

## Examples

```shell
some stream | cliplot
```
Plots data from an input stream, parsing for `\$([+|-]?\d*\.?\d*)\$` (a float surrounded by $)

---

```shell
 python3 print_with_delta.py 0.0032 | cliplot -r '\$([+|-]?\d*\.?\d*)\$' -r '%([+|-]?\d*\.?\d*)%' -c test.csv
```
Plots data from a [python script](test_assets/print_with_delta.py), with channel 0 eliminated by $$ and channel 1 eliminated
by %%. Also log this data to a CSV file called test.csv.

## Installation

### From source
```shell
cargo install cliplot
```
or with this repo
```shell
cargo build --release
# Binary is in target/release
target/release/cliplot --help
```

### Binaries
The GitHub releases tab should have debian archives and portable Windows builds.
