[![Crates.io](https://img.shields.io/crates/v/tree-rs.svg)](https://crates.io/crates/tree-rs)

# Tree-rs

Tree-rs tries to create a cross-platform alternative to the unix `tree` command.
The goal is to be compatible with its command line arguments.

`tree-rs` has been tested on Linux and Windows 7.

## Example output

    .
    ├── Cargo.lock
    ├── Cargo.toml
    ├── README.md
    ├── src
    │   └── main.rs
    └── test
        └── file

## Installation

```
git clone https://github.com/sighol/tree-rs
cargo install
```

## Performance

The performance is not as good as the windows `tree` or linux `tree` commands,
but it prints way faster than I can read.

From small benchmarks, it looks like it is about 2-10 times slower than linux
`tree`.
