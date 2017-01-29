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

From crates.io
```
cargo install tree-rs
```

From github
```
git clone https://github.com/sighol/tree-rs
cargo install
```

## Performance

On linux it is a bit faster than the `tree` command.
Based on a simple benchmark (see `timer.py`), it is about 20% faster.

