[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/tree-rs.svg)](https://crates.io/crates/tree-rs)
[![Rust CI](https://github.com/sighol/tree-rs/actions/workflows/rust-ci.yaml/badge.svg)](https://github.com/sighol/tree-rs/actions/workflows/rust-ci.yaml)

# Tree-rs

Cross-platform alternative to the unix `tree` command.

`tree-rs` has been tested on Linux and Windows 7.

## Example output

    .
    ├── Cargo.lock
    ├── Cargo.toml
    ├── README.md
    ├── src
    │   ├── filter.rs
    │   ├── main.rs
    │   └── pathiterator.rs
    └── timer.py

    1 directories, 7 files

## Installation

```bash
cargo install tree-rs
# or
cargo install --git https://github.com/sighol/tree-rs
```
