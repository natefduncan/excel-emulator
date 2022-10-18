# Excel Emulator

Convert Excel workbooks into Rust logic. 

## Installation

`cargo install excel-emulator` for the an `excel` binary.

## Usage

```
excel-emulator 0.1.0

USAGE:
    excel [OPTIONS] <PATH> [SUBCOMMAND]

ARGS:
    <PATH>    

OPTIONS:
    -d, --debug       Print cell-level calculation information
    -h, --help        Print help information
    -p, --progress    Display load and calculation progress bar
    -V, --version     Print version information

SUBCOMMANDS:
    calculate    Calculate a range
    deps         Print deps in DotGraph format
    get          Get a range
    help         Print this message or the help of the given subcommand(s)
    load         Load workbook
    order        Print cell calculation order
    sheets       Print workbook sheets
```

## Demo

![example](https://user-images.githubusercontent.com/30030731/196530970-3d3d2e12-049c-406e-abbb-a8b98532f542.gif)
