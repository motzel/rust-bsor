# rust-bsor

[BS Open Replay](https://github.com/BeatLeader/BS-Open-Replay) Rust parser

Disclaimer: This is my Rust learning project, so expect bugs and non-idomatic code

## Known limitations

Version 0.1.1 does not support replays saved out of specification by a very old version of the Beat Leader mod (incorrect utf8 string encoding).

## Install

Run following Cargo command in your project directory

```sh
cargo add bsor
```

Or add the following line to your ``[dependencies]`` section of the ``Cargo.toml``:

```toml
bsor = "0.1.1"
```

## Usage

```rust
use bsor::replay::Replay;
use std::fs::File;
use std::io::BufReader;

fn main() {
    let mut br = BufReader::new(File::open("test.bsor").unwrap());

    let replay = Replay::load(&mut br).unwrap();

    println!("{:#?}", replay);
}

```